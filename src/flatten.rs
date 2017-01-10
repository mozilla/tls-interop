fn flatten_sub(mat: &Vec<Vec<Vec<String>>>,
               index: usize,
               so_far: &Vec<String>,
               output: &mut Vec<Vec<String>>) {
    if index == mat.len() {
        output.push(so_far.clone());
        return;
    }

    // nxt is now pointing to Vec<Vec<String>>, so iterate over
    // that vector, flattening each string into so_far and then
    // pass it down to the next set of variables.
    for args in mat.get(index).unwrap() {
        let mut so_far_inner = so_far.clone();
        for arg in args {
            so_far_inner.push(arg.clone());
        }

        flatten_sub(mat, index + 1, &so_far_inner, output);
    }
}

pub fn flatten(mat: &Vec<Vec<Vec<String>>>) -> Vec<Vec<String>> {
    let so_far = vec![];
    let mut result = vec![];

    flatten_sub(mat, 0, &so_far, &mut result);
    return result;
}
