// --- Should warn: all impls return the same single-field variant ---

enum Output {
    Text(String),
    Number(i64),
}

trait Render {
    fn render(&self) -> Output;
}

struct A;
impl Render for A {
    fn render(&self) -> Output {
        Output::Text("hello".to_string())
    }
}

struct B;
impl Render for B {
    fn render(&self) -> Output {
        if true {
            return Output::Text("early".to_string());
        }
        Output::Text("world".to_string())
    }
}

// --- Should NOT warn: different variants across impls ---

enum Shape {
    Circle(f64),
    Square(f64),
}

trait Draw {
    fn shape(&self) -> Shape;
}

struct C;
impl Draw for C {
    fn shape(&self) -> Shape {
        Shape::Circle(1.0)
    }
}

struct D;
impl Draw for D {
    fn shape(&self) -> Shape {
        Shape::Square(2.0)
    }
}

// --- Should NOT warn: pub trait ---

pub enum Color {
    Red(u8),
    Blue(u8),
}

pub trait Paint {
    fn color(&self) -> Color;
}

struct E;
impl Paint for E {
    fn color(&self) -> Color {
        Color::Red(255)
    }
}

// --- Should NOT warn: variant has two fields ---

enum Pair {
    Both(i32, i32),
    Neither,
}

trait MakePair {
    fn pair(&self) -> Pair;
}

struct F;
impl MakePair for F {
    fn pair(&self) -> Pair {
        Pair::Both(1, 2)
    }
}

// --- Should NOT warn: no impls ---

enum Status {
    Ok(String),
    Err(String),
}

trait Check {
    fn status(&self) -> Status;
}

fn main() {}
