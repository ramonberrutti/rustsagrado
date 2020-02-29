use std::env;
use serenity::{
    model::{
        channel::Message,
        gateway::Ready,
    },
    prelude::*,
};
extern crate redis;
use redis::Commands;

extern crate rand;
use rand::Rng;

struct Handler {
    r: redis::Client,
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        if !msg.content.starts_with("!") {
            return;
        }

        let user_id = format!("user:{}", msg.author.id);
        let channel_id = format!("channel:{}", msg.channel_id);
        let mut conn = self.r.get_connection().unwrap();

        let args: Vec<&str> = msg.content.splitn(2, char::is_whitespace).collect();

        let response = match args[0].to_lowercase().as_str() {
            "!pedir" => {
                match args.len() {
                    0 => String::from("pedido invalido"),
                    _ => {
                        let save_value = format!("{}: {}", msg.author.name, args[1]);
                        let _: () = conn.hset(channel_id, user_id, save_value).unwrap();
                        String::from("obrigado")
                    },
                }
            },
            "!cancelar" => {
                let _: () = conn.hdel(channel_id, user_id).unwrap();
                String::from("cancelado")
            },
            "!finalizar" => {
                let vals: Vec<String> = conn.hvals(channel_id.to_string()).unwrap();
                match vals.len() {
                    0 => String::from("no tem pedidos"),
                    _ => {
                        let _: () = conn.del(channel_id).unwrap();
                        vals.join("\n")
                    },
                }
            },
            "!pedidos" => {
                let vals: Vec<String> = conn.hvals(channel_id).unwrap();
                match vals.len() {
                    0 => String::from("no tem pedidos"),
                    _ => vals.join("\n"),
                }
            },
            "!sortear" => {
                let keys: Vec<String> = conn.hkeys(channel_id).unwrap();
                match keys.len() {
                    0 => String::from("no tem pedidos"),
                    _ => {
                        let mut rng = rand::thread_rng();
                        let selected_id = rng.gen_range(0, keys.len());
                        let selected_id = keys[selected_id].replacen("user:", "", 1);
                        format!("<@{}> foi sorteado", selected_id)
                    },
                }
            },
            _ => String::new(),
        };

        if response.len() > 0 {
            if let Err(why) = msg.channel_id.say(&ctx, response) {
                println!("Error sending message: {:?}", why);
            }
        }

        //     if let Err(why) = msg.react(&ctx, "😄") {
        //         println!("Error sending reaction: {:?}", why);
        //     }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        println!("https://discordapp.com/api/oauth2/authorize?client_id={}&scope=bot&permissions=215104", ready.user.id);
    }
}


fn main() {
    println!("Hello, I'm RustSagrado!");

    let redis_addr = env::var("REDIS_ADDR")
        .expect("Expected a redis address");
    let redis_client = redis::Client::open(redis_addr).unwrap();

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = Client::new(&token, Handler{r: redis_client}).expect("Error creating client");
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why)
    }
}
