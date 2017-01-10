// Test the flattener
#[test]
fn flatten_unittest() {
    use flatten::flatten;

    let mut mat = vec![];
    let mut list1 = vec![];
    list1.push(vec![String::from("l1.1.0")]);
    list1.push(vec![String::from("l1.2.0"), String::from("l1.2.1")]);
    mat.push(list1);
    let mut list2 = vec![];
    list2.push(vec![String::from("l2.1.0")]);
    list2.push(vec![String::from("l2.2.1")]);
    list2.push(vec![String::from("l2.2.2")]);
    mat.push(list2);

    let flat = flatten(&mat);
    assert_eq!(6, flat.len());
}
