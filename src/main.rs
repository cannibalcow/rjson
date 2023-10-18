use rjson::{Json, JsonParser};

fn main() {
    //    let j = r#"{ "age": 3, "name": "heldt" }"#.to_string();
    let j = r#"
    {
        "age": 3,
        "sub": {
            "car": true,
            "isFast": false,
            "superSub": {
                "megafis": -123,
                "korv": "bullens"
            }
        }
    }"#
    .to_string();

    println!("{}", j);

    let mut json = Json::new(j);

    let result = json.parse();

    println!("{:?}", result.unwrap());
}
