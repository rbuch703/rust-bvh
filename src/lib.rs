mod primitives;
mod obj;

pub use primitives::{Bounded2D, BoundingBox, Bounds, Pos2, Pos3, Triangle};
pub use obj::parse_obj;

const MAX_ITEMS_PER_NODE: usize = 32;

enum Axis {
    X,
    Y,
}

pub struct KDTreeNodeSubdivision<T>
where
    T: Bounded2D,
{
    axis: Axis,
    position: f64,
    child_nodes: [KDTreeNode<T>; 2],
}

struct KDTreeNode<T>
where
    T: Bounded2D,
{
    children: Vec<T>,
    subdivision: Option<Box<KDTreeNodeSubdivision<T>>>,
}

impl<T> KDTreeNode<T>
where
    T: Bounded2D+CairoDrawable,
{
    fn new(items: Vec<T>) -> Self {
        if items.len() < MAX_ITEMS_PER_NODE {
            return KDTreeNode {
                children: items,
                subdivision: None,
            };
        }

        let bounds = Bounds::from_items(&items).expect("non-empty list");

        if bounds.x.size() > bounds.y.size() {
            let mid_x = (bounds.x.min + bounds.x.max) / 2.0;
            let mut items_here = Vec::new();
            let mut items_negative = Vec::new();
            let mut items_positive = Vec::new();

            for item in items {
                let bounds = item.bounds();
                if bounds.x.max < mid_x {
                    // entirely in negative half-space
                    items_negative.push(item);
                } else if bounds.x.min >= mid_x {
                    // entirely in positive half-space
                    items_positive.push(item);
                } else {
                    // overlaps with dividing line
                    items_here.push(item)
                }
            }
            KDTreeNode {
                children: items_here,
                subdivision: Some(Box::new(KDTreeNodeSubdivision {
                    axis: Axis::X,
                    position: mid_x,
                    child_nodes: [
                        KDTreeNode::new(items_negative),
                        KDTreeNode::new(items_positive),
                    ],
                })),
            }
        } else {
            let mid_y = (bounds.y.min + bounds.y.max) / 2.0;
            let mut items_here = Vec::new();
            let mut items_negative = Vec::new();
            let mut items_positive = Vec::new();

            for item in items {
                let bounds = item.bounds();
                if bounds.y.max < mid_y {
                    // entirely in negative half-space
                    items_negative.push(item);
                } else if bounds.y.min >= mid_y {
                    // entirely in positive half-space
                    items_positive.push(item);
                } else {
                    // overlaps with dividing line
                    items_here.push(item)
                }
            }
            KDTreeNode {
                children: items_here,
                subdivision: Some(Box::new(KDTreeNodeSubdivision {
                    axis: Axis::Y,
                    position: mid_y,
                    child_nodes: [
                        KDTreeNode::new(items_negative),
                        KDTreeNode::new(items_positive),
                    ],
                })),
            }
        }
    }

    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::from_items(&self.children)
    }

    fn bounding_box_recursive(&self) -> BoundingBox {
        let mut res = BoundingBox::from_items(&self.children);
        if let Some(sub) = &self.subdivision {
            res |= sub.child_nodes[0].bounding_box_recursive();
            res |= sub.child_nodes[1].bounding_box_recursive();
        }
        res
    }

    fn print_stats_recursive(&self, depth: usize) {
        let spaces : String = (0..depth*4).map(|_| ' ').collect();

        println!("{}{}", spaces, self.children.len());
        if let Some(sub) = &self.subdivision {
            sub.child_nodes[0].print_stats_recursive(depth+1);
            sub.child_nodes[1].print_stats_recursive(depth+1);
        }
    }

    fn draw_recursive(&self, context: &cairo::Context, depth: usize) {
        for item in &self.children {
           context.set_line_width(0.1);
           context.set_source_rgb(1.0, 0.0, 0.0);
           item.draw(context);
        }
   
        if let Some(sub) = &self.subdivision {
            sub.child_nodes[0].draw_recursive(context, depth + 1);
            sub.child_nodes[1].draw_recursive(context, depth + 1);
   
           let bounds = self.bounding_box_recursive();
           if let BoundingBox::Valid(bounds) = bounds {
               match sub.axis {
                   Axis::X => {
                       context.move_to(sub.position, bounds.y.min);
                       context.line_to(sub.position, bounds.y.max);
                   }
                   Axis::Y => {
                       context.move_to(bounds.x.min, sub.position);
                       context.line_to(bounds.x.max, sub.position);
                   }
               }
               context.set_line_width(1.0 - 0.05 * (depth as f64));
               context.set_source_rgb(0.0, 1.0 - 0.05 * (depth as f64), 0.05 * (depth as f64));
               context.stroke().expect("valid stroke");
           }
           //draw_bounds(context, node.bounding_box_recursive(), 0.0, 1.0 - (depth as f64)/10.0, 0.5);
       }
   }
}

pub trait CairoDrawable {
    fn draw(&self, context: &cairo::Context);
}

impl CairoDrawable for Triangle<Pos2> {
    fn draw(&self, context: &cairo::Context) {
        context.move_to(self.v1.x, self.v1.y);
        context.line_to(self.v2.x, self.v2.y);
        context.line_to(self.v3.x, self.v3.y);
        context.line_to(self.v1.x, self.v1.y);
        context.stroke().expect("stroke");
    }
}

pub struct OctreeNode<T>
where
    T: Bounded2D+CairoDrawable{
    pub bounds : BoundingBox,
    pub items: Vec<T>,
    pub children: Option<Box<[OctreeNode<T>; 4]>>
}

impl <T> OctreeNode<T>
where T: Bounded2D+CairoDrawable {
    pub fn new(items: Vec<T>) -> Self {

        let bounds = BoundingBox::from_items(&items);

        if let BoundingBox::Valid(rect) = bounds {
            if items.len() > MAX_ITEMS_PER_NODE {
                let mid_x = (rect.x.min + rect.x.max) / 2.0;
                let mid_y = (rect.y.min + rect.y.max) / 2.0;

                let mut items_tl = Vec::<T>::new();
                let mut items_tr = Vec::<T>::new();
                let mut items_bl = Vec::<T>::new();
                let mut items_br = Vec::<T>::new();
                let mut items_crossing_mid = Vec::<T>::new();
                for item in items {
                    let bounds = item.bounds();
                    if bounds.x.max < mid_x {
                        // entirely in left half
                        if bounds.y.max < mid_y {
                            // entirely in top-left corner
                            items_tl.push(item);
                        } else if bounds.y.min >= mid_y{
                            // entirely in bottom-left corner
                            items_tr.push(item);
                        } else {
                            items_crossing_mid.push(item);
                        }
                    } else if bounds.x.min >= mid_x {
                        // entirely in right half
                        if bounds.y.max < mid_y {
                            //entirely in top-right corner
                            items_bl.push(item)
                        } else if bounds.y.min >= mid_y {
                            // entirely in bottom-right corner
                            items_br.push(item)
                        } else {
                            items_crossing_mid.push(item)
                        }
                    } else {
                        items_crossing_mid.push(item)
                    }
                }

                OctreeNode::<T>{
                    bounds,
                    items: items_crossing_mid,
                    children: Some(Box::new([
                        OctreeNode::<T>::new(items_tl),
                        OctreeNode::<T>::new(items_tr),
                        OctreeNode::<T>::new(items_bl),
                        OctreeNode::<T>::new(items_br)
                    ]))
                }
            } else {
                OctreeNode::<T>{
                    bounds,
                    items,
                    children: None
                }
            }

        } else {
            OctreeNode::<T>{
                bounds, items, children:None
            }
        }
    }
/*
    fn shrink_bounds_recursive(&mut self) {
        self.bounds = BoundingBox::Empty;
        for item in &self.items {
            self.bounds = self.bounds | item.bounds();
        }

        if let Some(ref mut children) = self.children {
            for child in children.iter_mut() {
                child.shrink_bounds_recursive();
                self.bounds = self.bounds | child.bounds;
            }
        }
    }
*/
    pub fn draw_recursive(&self, context: &cairo::Context, depth :usize) {
        context.set_line_width(0.03);
        context.set_source_rgb(0.0, 1.0 - (depth as f64) / 10.0, (depth as f64) / 10.0);
        self.bounds.draw(context);

        if let Some(children) = &self.children {
            for child in children.iter() {
                child.draw_recursive(context, depth+1);
            }
        } else {
            for item in &self.items {
                context.set_line_width(0.1);
                context.set_source_rgb(1.0, 0.0, 0.0);
                item.draw(context);
            }
    
        }

    }
}

impl CairoDrawable for BoundingBox {
    fn draw(&self, context: &cairo::Context) {
        if let BoundingBox::Valid(rect) = self {
            context.rectangle(rect.x.min, rect.y.min, rect.x.size(), rect.y.size());
            context.stroke().unwrap();
        }
    }
}