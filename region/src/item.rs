pub enum Stackable {
    Arrow,
    Bomb,
}

pub enum Tool {
    Lantern,
    Glider,
    GrapplingHook,
    Shield,
}

pub enum Food {
    Apple,
    Bread,
    Beef,
}

pub enum Potion {
    Health,
    Damage,
    Mystery,
}

pub enum Weapon {
    Dagger,
    Sword,
    Bow,
}

pub enum Item {
    Stackable {
        number: u8,
        variant: Stackable,
    },
    Tool {
        damage: u8,
        quality: u8,
        variant: Tool,
    },
    Food {
        energy: u8,
        variant: Food,
    },
    Potion {
        effect: u8,
        variant: Potion,
    },
    Weapon {
        damage: u8,
        strength: u8,
        variant: Weapon,
    },
}
