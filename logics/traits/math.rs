use primitive_types::U256;

pub trait PercentMath {
    fn percent_mul(&self, percentage: U256) -> U256;
    fn percent_div(&self, percentage: U256) -> U256;
}

pub trait WadRayMath {
    fn ray(&self) -> U256;
    fn wad(&self) -> U256;
    fn half_ray(&self) -> U256;
    fn half_wad(&self) -> U256;
    fn wad_mul(&self, b: U256) -> U256;
    fn wad_div(&self, b: U256) -> U256;
    fn ray_mul(&self, b: U256) -> U256;
    fn ray_div(&self, b: U256) -> U256;
    fn ray_to_wad(&self) -> U256;
    fn wad_to_ray(&self) -> U256;
}
