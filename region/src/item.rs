#[derive(Debug, Clone)]
pub enum Stackable {
    Arrow,
    Bomb,
}

#[derive(Debug, Clone)]
pub enum Tool {
    Lantern,
    Glider,
    GrapplingHook,
    Shield,
}

#[derive(Debug, Clone)]
pub enum Food {
    Apple,
    Bread,
    Beef,
}

#[derive(Debug, Clone)]
pub enum Potion {
    Health,
    Damage,
    Mystery,
}

#[derive(Debug, Clone)]
pub enum Weapon {
    Dagger,
    Sword,
    Bow,
}

#[derive(Debug, Clone)]
pub enum Item {
    Stackable { number: u8, variant: Stackable },
    Tool { damage: u8, quality: u8, variant: Tool },
    Food { energy: u8, variant: Food },
    Potion { effect: u8, variant: Potion },
    Weapon { damage: u8, strength: u8, variant: Weapon },
}
