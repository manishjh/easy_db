use actix_web::web::{Json, Path};
use actix_web::{Error, HttpResponse};

use crate::KeyValue;

pub async fn get(path: Path<String>) -> Result<String, Error> {
    log::info!("get -> {}", path.as_str());

    //send to a replica don't send to master for reads.
    //will need to reply from replica with hpc >= max(last client read hpc,last client write hpc)
    //how will it know which replica is at what hpc?
    //indentify client from clientID, add last read hpc time to list.

    Ok("response".to_string())
}

pub async fn post(data: Json<KeyValue>) -> Result<HttpResponse, Error> {
    log::info!("adding -> {}: {}", data.key, data.value);

    //send to master node.
    //master responds with an "OK"
    //identify client from clientID, add last write hpc time to list.

    Ok(HttpResponse::Created().finish())
}
