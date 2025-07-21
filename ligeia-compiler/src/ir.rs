use eisheth::package::Package;

pub fn print_to_files(name: &str, package: &Package) {
    std::fs::create_dir_all(format!("./output/{name}/modules/")).unwrap();

    for (module_name, raw_ir) in package.ir_per_module() {
        std::fs::write(format!("./output/{name}/modules/{module_name}.ll"), raw_ir).unwrap();
    }

    std::fs::write(format!("./output/{name}/linked.ll"), package.final_ir()).unwrap();
}
