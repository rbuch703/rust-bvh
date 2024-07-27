extern crate cairo;
extern crate rand;

use rust_bvh::{parse_obj, BoundingBox, Bounds, OctreeNode, Pos2, Pos3, Triangle};

fn map_vertices<T, U,F>(src : &[Triangle<T>], trans: F) -> Vec<Triangle<U>>
where F: Fn(&T)->U {
    src.iter().map(|Triangle { v1, v2, v3 }|{
        Triangle::new( trans(v1), trans(v2), trans(v3))
    }).collect()
}

fn main() {
    let bunny = parse_obj("bunny.obj").expect("reading input");

    let bunny = map_vertices(&bunny, |Pos3{x,y,z:_}| Pos2 { x:*x, y:*y });
    let bounds = Bounds::from_items(&bunny).expect("bounds");

    let bunny = map_vertices(&bunny, |Pos2 { x, y }|
         Pos2{
            x: (x - bounds.x.min) / bounds.x.size()*100.0,
            y: (y - bounds.y.min) / bounds.y.size()*100.0
    });
    println!("Bunny has {} faces", bunny.len());
    let surface = cairo::PdfSurface::new(100.0, 100.0, "out.pdf").unwrap();
    let context = cairo::Context::new(&surface).unwrap();

    /*
    use rand::Rng;
    let mut rng = rand::XorShiftRng::new_unseeded();
    const NUM_POINTS: usize = 70001;
    let points: Vec<_> = (0..NUM_POINTS)
        .map(|_| (rng.next_f64() * 100.0, rng.next_f64() * 100.0))
        .collect();
    */

    //let root = KDTreeNode::new(bunny);
    let root = OctreeNode::new(bunny);

    if let BoundingBox::Valid(bbox) = root.bounds {
        println!("{bbox:?}");
    }

    root.draw_recursive(&context, 0);
}
