use eisheth::package::Package;

pub fn print_to_files(package: &Package) {
    std::fs::create_dir_all("./output/modules/").unwrap();

    for (module_name, raw_ir) in package.ir_per_module() {
        std::fs::write(format!("./output/modules/{module_name}.ll"), raw_ir).unwrap();
    }

    std::fs::write("./output/linked.ll", package.final_ir()).unwrap();
}
