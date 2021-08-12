use systems_core::{ Observation, Storage, ComponentAccess, MutationObserver, get_component_mut };


// TODO possible optimzization: 
// organize damages guys into a list
#[derive(Clone, Copy, Debug)]
pub struct HpInfo {
    current_hp : u16,
    gathered_damage : u16,
    // NOTE this is the only field needed 
    // for the optimization
    // next : Option<usize>,

}

impl HpInfo {
    pub fn new(starting_hp : u16) -> Self {
        HpInfo {
            current_hp : starting_hp,
            gathered_damage : 0,
        }
    }

    // NOTE if the detection for bullets go bad, this
    // method might be the culprit :)
    #[inline]
    pub fn is_alive(&self) -> bool { self.current_hp > 0 }

    #[inline]
    pub fn is_damaged(&self) -> bool { self.gathered_damage > 0 }

    #[inline]
    pub fn damage(&mut self, dmg : u16) {
        self.gathered_damage += dmg;
    }

    #[inline]
    pub fn hp(&self) -> u16 { self.current_hp }

    #[inline]
    pub unsafe fn hp_mut(&mut self) -> &mut u16 { &mut self.current_hp }
}

pub struct HpSystem {}

impl HpSystem {
    pub fn new() -> Self {
        HpSystem {}
    }

    pub fn update<Host, Obs>(
        &mut self,
        host : &mut Observation<Obs, Host>,
    ) 
    where
        Host : Storage,
        Obs : MutationObserver<Host>,
        Host::Object : ComponentAccess<HpInfo>,
    {
        host.mutate_each(|obj, _| {
            let hp_info = get_component_mut::<HpInfo, _>(obj);
            hp_info.current_hp = hp_info.current_hp.saturating_sub(hp_info.gathered_damage);
            hp_info.gathered_damage = 0;
        })
    }
}
