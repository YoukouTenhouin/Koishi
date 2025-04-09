use uuid::Uuid;

pub(crate) fn main() {
    let uuid = Uuid::now_v7();

    println!("{}", uuid.simple());
}
