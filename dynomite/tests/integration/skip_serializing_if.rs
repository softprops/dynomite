use dynomite::{attr_map, Attributes, Item};

#[derive(Debug, Item)]
struct Blackjack {
    #[dynomite(partition_key)]
    card: String,

    #[dynomite(skip_serializing_if = "is_zero")]
    deck: u32,
    gamer: Gamer,
}

fn is_zero(&val: &u32) -> bool {
    val == 0
}

#[derive(Debug, Attributes)]
struct Gamer {
    #[dynomite(skip_serializing_if = "String::is_empty")]
    name: String,

    // verify that auto-deref coertions work
    #[dynomite(skip_serializing_if = "str::is_empty")]
    surname: String,

    label: String,
}

#[test]
fn smoke_test() {
    let item = Blackjack {
        card: "ace".to_owned(),
        deck: 0,
        gamer: Gamer {
            name: "".to_owned(),
            surname: "Fish".to_owned(),
            label: "".to_owned(),
        },
    };

    let attrs: dynomite::Attributes = item.into();

    let expected = attr_map! {
        "card" => "ace".to_owned(),
        "gamer" => attr_map! {
            "surname" => "Fish".to_owned(),
            "label" => "".to_owned()
        }
    };

    assert_eq!(attrs, expected);
}
