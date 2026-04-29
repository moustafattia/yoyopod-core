load("@crates//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_test")

def yoyopod_rust_host_binary(name, srcs, crate_root = "src/main.rs", **kwargs):
    rust_binary(
        name = name,
        crate_root = crate_root,
        srcs = srcs,
        aliases = aliases(),
        deps = all_crate_deps(normal = True),
        proc_macro_deps = all_crate_deps(proc_macro = True),
        edition = "2021",
        visibility = ["//visibility:public"],
        **kwargs
    )

def yoyopod_rust_host_test(name, crate, **kwargs):
    rust_test(
        name = name,
        crate = crate,
        aliases = aliases(
            normal_dev = True,
            proc_macro_dev = True,
        ),
        deps = all_crate_deps(normal_dev = True),
        proc_macro_deps = all_crate_deps(proc_macro_dev = True),
        visibility = ["//visibility:public"],
        **kwargs
    )
