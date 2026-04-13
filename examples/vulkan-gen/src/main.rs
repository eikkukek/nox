pub mod vk;
//mod commands;
mod macros;

fn main() {
    let mut id2 = vk::PhysicalDevicePresentId2FeaturesKHR::default();
    let mut vk14 = vk::PhysicalDeviceVulkan14Features::default();
    let mut features = vk::PhysicalDeviceFeatures2
        ::default()
        .push_next(&mut vk14)
        .push_next(&mut id2);
    for s in vk::chain_iter(&mut features) {
        println!("{}", s.s_type);
    }
}
