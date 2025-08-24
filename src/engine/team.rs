    #[derive(Debug, Clone)]
    pub struct Team {
        pub name: String,
        pub required_positions: Vec<Position>,
    }

    #[derive(Debug, Clone)]
    pub struct Position {
        pub qualification: String,
        pub count: usize,
    }