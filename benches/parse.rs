use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fnpm_semver::{PackageReq, Version, VersionSpec};

fn bench_version_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("version_parse");
    group.bench_function("simple", |b| {
        b.iter(|| Version::parse(black_box("1.2.3")))
    });
    group.bench_function("prerelease", |b| {
        b.iter(|| Version::parse(black_box("1.0.0-alpha.1")))
    });
    group.bench_function("build_metadata", |b| {
        b.iter(|| Version::parse(black_box("1.0.0+build.123")))
    });
    group.bench_function("full", |b| {
        b.iter(|| Version::parse(black_box("1.0.0-beta.2+build.456")))
    });
    group.finish();
}

fn bench_version_spec_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("version_spec_parse");
    group.bench_function("caret", |b| {
        b.iter(|| VersionSpec::parse(black_box("^1.2.3")))
    });
    group.bench_function("tilde", |b| {
        b.iter(|| VersionSpec::parse(black_box("~1.2.3")))
    });
    group.bench_function("range", |b| {
        b.iter(|| VersionSpec::parse(black_box(">=1.0.0 <2.0.0")))
    });
    group.bench_function("or", |b| {
        b.iter(|| VersionSpec::parse(black_box("^1.0.0 || ^2.0.0")))
    });
    group.bench_function("star", |b| {
        b.iter(|| VersionSpec::parse(black_box("*")))
    });
    group.bench_function("exact", |b| {
        b.iter(|| VersionSpec::parse(black_box("1.2.3")))
    });
    group.finish();
}

fn bench_package_req_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("package_req_parse");
    group.bench_function("simple", |b| {
        b.iter(|| PackageReq::parse(black_box("lodash@^4.17.0")))
    });
    group.bench_function("scoped", |b| {
        b.iter(|| PackageReq::parse(black_box("@babel/core@^7.0.0")))
    });
    group.bench_function("no_version", |b| {
        b.iter(|| PackageReq::parse(black_box("express")))
    });
    group.finish();
}

fn bench_matches(c: &mut Criterion) {
    let mut group = c.benchmark_group("matches");

    let spec = VersionSpec::parse("^1.0.0").unwrap();
    let req = PackageReq::parse("lodash@>=1.0.0 <3.0.0").unwrap();

    let v_match = Version::parse("1.5.0").unwrap();
    let v_no_match = Version::parse("2.0.0").unwrap();

    group.bench_function("spec_hit", |b| {
        b.iter(|| spec.matches(black_box(&v_match)))
    });
    group.bench_function("spec_miss", |b| {
        b.iter(|| spec.matches(black_box(&v_no_match)))
    });
    group.bench_function("req_hit", |b| {
        let v = Version::parse("4.17.21").unwrap();
        b.iter(|| req.matches(black_box(&v)))
    });
    group.bench_function("req_miss", |b| {
        b.iter(|| req.matches(black_box(&v_no_match)))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_version_parse,
    bench_version_spec_parse,
    bench_package_req_parse,
    bench_matches,
);
criterion_main!(benches);