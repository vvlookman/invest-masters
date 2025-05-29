use invmst::api;
use strum::EnumMessage;
use tabled::settings::{Color, object::Columns};

#[derive(clap::Args)]
pub struct MastersCommand;

impl MastersCommand {
    pub async fn exec(&self) {
        let mut table_data: Vec<Vec<String>> = vec![];

        let masters = api::masters::list().await;
        for master in masters {
            let name = master.get_message().unwrap_or_default().to_string();
            let keys = master.get_serializations().join("/");
            table_data.push(vec![name, keys]);
        }

        let mut table = tabled::builder::Builder::from_iter(&table_data).build();
        table.modify(Columns::first(), Color::FG_GREEN);
        println!("{table}");
    }
}
