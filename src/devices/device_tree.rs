use device_tree::{DeviceTree, Node};

const DEVICE_TREE_MAGIC: u32 = 0xD00DFEED;
use ::alloc::{*, string::String, vec::Vec};
use byteorder::{LittleEndian, ByteOrder};

#[repr(C)]
struct DtbHeader {
    magic: u32,
    size: u32,
}

fn print(node: &Node, level: usize) {
    let mut indent = String::new();
    if level >= 2 {
        for _ in 1..level {
            indent.push_str("  ");
        }
        print!("{}", indent);
        print!("+-");
    }
    println!("{}", node.name);
    for (k, v) in &node.props {
        print!("{}", indent);
        print!("| {}: ",k);
        if k == "reg" {
            for (index, byte) in v.iter().enumerate() {
                if index % 4 == 0 {
                    print!(" ");
                }
                print!("{:02X?} ", byte);
            }
        } else if k == "device_type" {
            print!("{:?}", node.prop_str(k));
        } else if k == "compatible" {
            print!("{:?}", node.prop_str_list(k));
        } else {
            print!("{:?}", v);
        }
        println!("");
    }
    // log!("{}{}", indent, node.name);
    for child in node.children.iter() {
        print(child, level + 1);
    }
}

pub unsafe fn print_tree(dtb_pa: usize) {
    log!("Tree addr: {:p}", dtb_pa as *const u8);
    let header = &*(dtb_pa as *const DtbHeader);
    let magic = u32::from_be(header.magic);
    if magic == DEVICE_TREE_MAGIC {
        log!("Found device tree!");
        let size = u32::from_be(header.size);
        log!("size: {:p}", size as *const u8);
        // 拷贝数据，加载并遍历
        let data = core::slice::from_raw_parts(dtb_pa as *const u8, size as usize);
        if let Ok(dt) = DeviceTree::load(data) {
            print(&dt.root, 0);
        } else {
            panic!("Failed to load device tree, maybe not a device tree");
        }
        return;
    }
    panic!("Failed to load device tree");
}
