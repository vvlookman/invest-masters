use strum::IntoEnumIterator;

use crate::masters::Master;

pub async fn list() -> Vec<Master> {
    Master::iter().collect()
}
