default:basic crud relations

basic:
  cargo run --example=basic --features postgres,uuid,extra-traits --quiet -- --no-capture

crud:
  cargo run --example=crud --features postgres,uuid,extra-traits --quiet -- --no-capture

relations:
  cargo run --example=relations --features postgres,uuid,extra-traits --quiet -- --no-capture

