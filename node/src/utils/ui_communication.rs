use crate::utils::btc_errors::{NodeError, ConfigError};

pub enum UIWalletCommunicationProtocol {
    ChangeWallet(/* pubkey*/String, /* private key*/String),    //ui se tiene que fijar que las longitudes esten bien, ya sea en hexa o en base 58. La wallet dependiendo de la cantidad lo pasa a array, y cambia la wallet. 
    CreateTx(/* amount*/i64, /* fee*/i64, /*addres */),         //ui manda en distintas bases el adrress, se fijan las longitudes
    Update,  //la wallet devuelve, un struct, con el balance, unspent balance, pagina actual de tx, ultimos 5 o 10headers
    ObtainTxProof,
    ObtainTxBalance(i64),
    ObtainTxPendingBalance(i64),
    ObtainTxPage(Vec<String>), //p temporalmente como string, despues creamos el tipo de dato correspondiente
    NodeRunningError(NodeError),
    ConfigError(ConfigError),
}