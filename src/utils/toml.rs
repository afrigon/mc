use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::Table;

pub fn set_comment(table: &mut Table, key: &str, comments: &Vec<String>) {
    if let Some(mut k) = table.key_mut(key) {
        let mut output = String::new();

        for comment in comments {
            output.push_str(format!("# {}\n", comment).as_str());
        }

        k.leaf_decor_mut().set_prefix(output);
    }
}

pub fn tight_format(document: &mut DocumentMut) {
    tight_format_table(document.as_table_mut());
}

fn tight_format_table(table: &mut Table) {}

fn tight_format_item(item: &mut Item) {}
