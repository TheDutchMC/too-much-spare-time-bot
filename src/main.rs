use log::{error, info};
use std::process::exit;

mod config;

#[tokio::main]
async fn main() {
    init_logger();
    info!(
        "Starting {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    match main::main().await {
        Ok(_) => {}
        Err(e) => {
            error!("{:?}", e);
            exit(1);
        }
    };
}

fn init_logger() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var(
            "RUST_LOG",
            format!("{}=INFO", env!("CARGO_PKG_NAME")).replace("-", "_"),
        );
    }
    env_logger::init();
}

mod main {
    use crate::config::Config;
    use crate::main::migrations::apply_migrations;
    use anyhow::Result;
    use std::path::PathBuf;
    use structopt::StructOpt;

    mod migrations {
        use anyhow::Result;
        use mysql::PooledConn;
        use refinery::embed_migrations;

        embed_migrations!("./migrations");
        pub fn apply_migrations(conn: &mut PooledConn) -> Result<()> {
            crate::main::migrations::migrations::runner().run(conn)?;
            Ok(())
        }
    }

    #[derive(StructOpt)]
    struct Opts {
        #[structopt(parse(from_os_str), short, long)]
        config: PathBuf,
    }

    pub async fn main() -> Result<()> {
        let opts: Opts = Opts::from_args();
        let config = Config::new(&opts.config)?;
        let mysql_pool = config.mysql_connection()?;
        apply_migrations(&mut mysql_pool.get_conn()?)?;

        crate::serenity::init_serenity(config, mysql_pool).await?;

        Ok(())
    }
}

mod serenity {
    use crate::config::Config;
    use anyhow::Result;
    use log::debug;
    use mysql::prelude::Queryable;
    use mysql::{params, Row};
    use serenity::{
        async_trait,
        framework::standard::{
            macros::{command, group},
            Args, CommandResult, StandardFramework,
        },
        model::{channel::Message, gateway::Ready},
        prelude::*,
    };
    use std::sync::Arc;

    struct Data {
        pool: mysql::Pool,
        config: Config,
    }

    struct DataMapType;

    impl TypeMapKey for DataMapType {
        type Value = Arc<Data>;
    }

    #[group]
    #[commands(freetime)]
    struct General;

    pub async fn init_serenity(config: Config, pool: mysql::Pool) -> Result<()> {
        let prog_data = Arc::new(Data {
            pool,
            config: config.clone(),
        });

        let framework = StandardFramework::new()
            .configure(|c| {
                c.with_whitespace(true)
                    .prefix(&config.discord.prefix)
                    .ignore_bots(true)
                    .ignore_webhooks(true)
            })
            .group(&GENERAL_GROUP);

        let mut client = Client::builder(&config.discord.token)
            .event_handler(Handler {
                data: prog_data.clone(),
            })
            .framework(framework)
            .await?;

        {
            let mut data = client.data.write().await;
            data.insert::<DataMapType>(prog_data);
        }

        client.start().await?;
        Ok(())
    }

    struct Handler {
        data: Arc<Data>,
    }

    #[async_trait]
    impl EventHandler for Handler {
        async fn message(&self, ctx: Context, message: Message) {
            debug!("Message event received");

            let data = self.data.clone();
            let mut conn = data.pool.get_conn().expect("Opening MySQL connection");

            conn.exec_drop(
                "INSERT INTO messages (message_id, user_id) VALUES (:message_id, :user_id)",
                params! {
                    "message_id" => message.id.0,
                    "user_id" => message.author.id.0
                },
            )
            .expect("Inserting message record into database");

            let row: Row = conn
                .exec_first(
                    "SELECT COUNT(1) FROM messages WHERE user_id = :user_id",
                    params! {
                        "user_id" => message.author.id.0
                    },
                )
                .expect("Retrieving message count")
                .unwrap();
            let row_count: u64 = row.get(0).unwrap();

            let applicable_roles = self
                .data
                .config
                .roles
                .iter()
                .filter(|f| row_count >= f.messages)
                .map(|f| f.id)
                .collect::<Vec<_>>();

            if let Some(guild) = message.guild(&ctx).await {
                let mut member = guild
                    .member(&ctx, &message.author.id)
                    .await
                    .expect("Fetching guild member");
                for role_id in applicable_roles {
                    member
                        .add_role(&ctx, role_id)
                        .await
                        .expect("Adding role to Member");
                }
            }
        }

        async fn ready(&self, _: Context, _: Ready) {}
    }

    #[command]
    async fn freetime(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
        debug!("Freetime command called");

        let data_read = ctx.data.read().await;
        let data_lock = data_read
            .get::<DataMapType>()
            .expect("Missing DataMapType")
            .clone();
        let mut conn = data_lock.pool.get_conn()?;

        let user = match msg.mentions.first() {
            Some(u) => u,
            None => &msg.author,
        };

        let row: Row = conn
            .exec_first(
                "SELECT COUNT(1) FROM messages WHERE user_id = :user_id",
                params! {
                    "user_id" => user.id.0
                },
            )?
            .unwrap();
        let row_count: u64 = row.get(0).unwrap();

        msg.reply(
            ctx,
            format!("{} has sent {} message(s)", user.name, row_count),
        )
        .await?;

        Ok(())
    }
}
