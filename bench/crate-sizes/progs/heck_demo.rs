use heck::{ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToTitleCase, ToUpperCamelCase, ToTrainCase};
fn main() {
    // Inputs the compilers can't fold: n is 1 at run time.
    let n = std::env::args().len();
    let ss = ["HelloWorld", "hello_world", "XMLHttpRequest", "some-kebab-case", "  spaces and CAPS  ", "ÄpfelÜber", "v2Number10"];
    for s in ss[n - 1..].iter() {
        println!("{} | {} | {} | {} | {} | {} | {} | {}", s, s.to_snake_case(), s.to_kebab_case(), s.to_lower_camel_case(),
            s.to_upper_camel_case(), s.to_shouty_snake_case(), s.to_title_case(), s.to_train_case());
    }
}
