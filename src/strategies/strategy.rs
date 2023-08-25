use rocket::info;
use tokio::runtime::Runtime;
use uuid::Uuid;

pub trait Strategy: Send + Sync {
    fn uuid(&self) -> Uuid;
    fn start(&mut self, executor: &Runtime);
    fn startup_complete(&self) {
        info!("{} strategy instantiated.", std::any::type_name::<Self>());
    }
    fn get_strategy_name(&self) -> String {
        std::any::type_name::<Self>().to_string()
    }
}

//
// pub struct ConcreteStrategy {
//     uuid: Uuid,
// }
//
// impl ConcreteStrategy {
//     pub fn new() -> Self {
//         let strategy = Self {
//             uuid: Uuid::new(),
//         };
//         strategy.startup_complete();
//         strategy
//     }
// }
//
// impl Strategy for ConcreteStrategy {
//     fn uuid(&self) -> Uuid {
//         return self.uuid;
//     }
//
//     fn start(&self) {
//         // Implement the start method here
//     }
// }
