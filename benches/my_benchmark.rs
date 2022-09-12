use criterion::{criterion_group, criterion_main, Criterion};
use maplit::btreemap;
use polyvinyl_acetate::ortho::{Location, Ortho};

fn make_sample_ortho() -> Ortho {
    let abde = Ortho::new(1, 2, 4, 5);

    let bcef = Ortho::new(2, 3, 5, 6);

    let degh = Ortho::new(4, 5, 7, 8);

    let efhi = Ortho::new(5, 6, 8, 9);

    // 1 2 3
    // 4 5 6
    // 7 8 9

    let abcdef = Ortho::zip_over(
        &abde,
        &bcef,
        &btreemap! {
            3 => 2,
            5 => 4
        },
        3,
    );

    let defghi = Ortho::zip_over(
        &degh,
        &efhi,
        &btreemap! {
            6 => 5,
            8 => 7
        },
        6,
    );

    let o = Ortho::zip_over(
        &abcdef,
        &defghi,
        &btreemap! {
            5 => 2,
            7 => 4
        },
        7,
    );
    o
}

pub fn vocabulary_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho vocabulary", |b| b.iter(|| o.get_vocabulary()));
}

pub fn hop_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho hop", |b| b.iter(|| o.get_hop()));
}

pub fn origin_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho origin", |b| b.iter(|| o.get_origin()));
}

pub fn contents_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho contents", |b| b.iter(|| o.get_contents()));
}

pub fn base_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho base", |b| b.iter(|| o.is_base()));
}

pub fn origin_phrase_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho origin phrase success", |b| {
        b.iter(|| o.origin_has_full_length_phrase(&vec![1, 4, 7]))
    });
}

pub fn origin_fail_phrase_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho origin phrase failure", |b| {
        b.iter(|| o.origin_has_full_length_phrase(&vec![1, 4, 7, 100]))
    });
}

pub fn hop_phrase_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho hop phrase success", |b| {
        b.iter(|| o.hop_has_full_length_phrase(&vec![2, 5, 8]))
    });
}

pub fn hop_fail_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho hop phrase failure", |b| {
        b.iter(|| o.hop_has_full_length_phrase(&vec![2, 5, 8, 100]))
    });
}

pub fn contents_phrase_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho contents phrase success", |b| {
        b.iter(|| o.contents_has_full_length_phrase(&vec![7, 8, 9]))
    });
}

pub fn contents_fail_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho contents phrase failure", |b| {
        b.iter(|| o.contents_has_full_length_phrase(&vec![7, 8, 9, 100]))
    });
}

pub fn bottom_right_corner_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("bottom right corner", |b| {
        b.iter(|| o.get_bottom_right_corner())
    });
}

pub fn get_dims_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho dims", |b| b.iter(|| o.get_dims()));
}

pub fn name_at_location_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    let loc = Location::singleton(2);
    c.bench_function("ortho name at location", |b| {
        b.iter(|| o.name_at_location(&loc))
    });
}

pub fn optional_name_at_location_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    let loc = Location::singleton(311);
    c.bench_function("ortho optional_name_at_location", |b| {
        b.iter(|| o.optional_name_at_location(&loc))
    });
}

pub fn get_dimensionality_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho get_dimensionality", |b| {
        b.iter(|| o.get_dimensionality())
    });
}

pub fn get_names_at_distance_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho get_names_at_distance", |b| {
        b.iter(|| o.get_names_at_distance(1))
    });
}

pub fn axis_length_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho axis_length", |b| b.iter(|| o.axis_length(2)));
}

pub fn to_vec_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho to_vec", |b| b.iter(|| o.to_vec()));
}

pub fn phrases_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho phrases", |b| b.iter(|| o.phrases(2)));
}

pub fn axis_of_change_between_names_for_hop_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho axis_of_change_between_names_for_hop", |b| {
        b.iter(|| o.axis_of_change_between_names_for_hop(4, 5))
    });
}

pub fn axes_of_change_between_names_for_contents_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho axes_of_change_between_names_for_contents", |b| {
        b.iter(|| o.axes_of_change_between_names_for_contents(3, 6))
    });
}

pub fn all_full_length_phrases_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho all_full_length_phrases", |b| {
        b.iter(|| o.all_full_length_phrases())
    });
}

pub fn origin_phrases_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho origin_phrases", |b| b.iter(|| o.origin_phrases()));
}

pub fn phrase_tail_summary_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho phrase_tail_summary", |b| {
        b.iter(|| o.phrase_tail_summary(2))
    });
}

pub fn phrase_head_summary_benchmark(c: &mut Criterion) {
    let o = make_sample_ortho();
    c.bench_function("ortho phrase_head_summary", |b| {
        b.iter(|| o.phrase_head_summary(2))
    });
}

criterion_group!(
    benches,
    vocabulary_benchmark,
    hop_benchmark,
    origin_benchmark,
    contents_benchmark,
    base_benchmark,
    origin_phrase_benchmark,
    origin_fail_phrase_benchmark,
    hop_phrase_benchmark,
    hop_fail_benchmark,
    contents_phrase_benchmark,
    contents_fail_benchmark,
    bottom_right_corner_benchmark,
    get_dims_benchmark,
    name_at_location_benchmark,
    optional_name_at_location_benchmark,
    get_dimensionality_benchmark,
    get_names_at_distance_benchmark,
    axis_length_benchmark,
    to_vec_benchmark,
    phrases_benchmark,
    axis_of_change_between_names_for_hop_benchmark,
    axes_of_change_between_names_for_contents_benchmark,
    all_full_length_phrases_benchmark,
    origin_phrases_benchmark,
    phrase_tail_summary_benchmark,
    phrase_head_summary_benchmark
);
criterion_main!(benches);
