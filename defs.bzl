load("@crates//:defs.bzl", "aliases", "all_crate_deps")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")

def yoyopod_rust_host_library(name, srcs, crate_root = "src/lib.rs", deps = None, **kwargs):
    if deps == None:
        deps = []
    rust_library(
        name = name,
        crate_root = crate_root,
        srcs = srcs,
        aliases = aliases(normal = True, proc_macro = True),
        deps = deps + all_crate_deps(normal = True),
        proc_macro_deps = all_crate_deps(proc_macro = True),
        edition = "2021",
        visibility = ["//visibility:public"],
        **kwargs
    )

def yoyopod_rust_host_binary(name, srcs, crate_root = "src/main.rs", deps = None, **kwargs):
    if deps == None:
        deps = []
    rust_binary(
        name = name,
        crate_root = crate_root,
        srcs = srcs,
        aliases = aliases(normal = True, proc_macro = True),
        deps = deps + all_crate_deps(normal = True),
        proc_macro_deps = all_crate_deps(proc_macro = True),
        edition = "2021",
        visibility = ["//visibility:public"],
        **kwargs
    )

def yoyopod_rust_host_integration_test(name, src, deps, extra_srcs = None, **kwargs):
    if extra_srcs == None:
        extra_srcs = []
    rust_test(
        name = name,
        crate_root = src,
        srcs = [src] + extra_srcs,
        aliases = aliases(
            normal = True,
            normal_dev = True,
            proc_macro = True,
            proc_macro_dev = True,
        ),
        deps = deps + all_crate_deps(normal = True, normal_dev = True),
        proc_macro_deps = all_crate_deps(proc_macro = True, proc_macro_dev = True),
        edition = "2021",
        visibility = ["//visibility:public"],
        **kwargs
    )

def yoyopod_rust_host_integration_tests(name, tests, deps, extra_srcs_by_test = None):
    if extra_srcs_by_test == None:
        extra_srcs_by_test = {}
    test_targets = []
    for test in tests:
        test_target = "%s_test" % test
        yoyopod_rust_host_integration_test(
            name = test_target,
            src = "tests/%s.rs" % test,
            deps = deps,
            extra_srcs = extra_srcs_by_test.get(test, []),
        )
        test_targets.append(test_target)
    native.test_suite(
        name = name,
        tests = test_targets,
    )
