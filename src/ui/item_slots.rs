use bevy::prelude::*;
use crate::items::{InventoryTrait, Item, Stack};
use super::{hotbar::HotBarSlot, ui_tex_map::{UiTextureMap, SLOT_SIZE_PERCENT}, Inventory, SelectedHotbarSlot};

pub struct ItemSlotPlugin;

impl Plugin for ItemSlotPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Dragging(None))
            .add_systems(Startup, setup_dragging_ui)
            .add_systems(OnEnter(Inventory), show_dragging_ui)
            .add_systems(OnExit(Inventory), hide_dragging_ui)
            .add_systems(Update, item_slot_click.run_if(in_state(Inventory)))
            .add_systems(Update, refresh_slot_items)
            .add_systems(Update, refresh_dragging_ui.run_if(in_state(Inventory)))
            .add_systems(Update, refresh_dragging_ui_pos.run_if(in_state(Inventory)))
            ;
    }
}

#[derive(Resource)]
struct Dragging(pub Option<(Stack, Entity)>);

#[derive(Component)]
struct DraggingNode;

#[derive(Component, Clone)]
pub struct UISlot(pub Entity, pub usize);

pub enum FurnaceSlot {
    Material,
    Fuel,
    Output
}

impl From<FurnaceSlot> for usize {
    fn from(value: FurnaceSlot) -> Self {
        value as usize
    }
}

impl From<usize> for FurnaceSlot {
    fn from(value: usize) -> Self {
        match value {
            0 => FurnaceSlot::Material,
            1 => FurnaceSlot::Fuel,
            2 => FurnaceSlot::Output,
            _ => unreachable!()
        }
    }
}

// TODO: If/When trait queries get adopted by Bevy (https://github.com/bevyengine/bevy/issues/15970)
// get rid of this enum and use a trait instead, item holding components will implement this trait
#[derive(Component)]
pub enum ItemHolder {
    Furnace { fuel: Stack, material: Stack, output: Stack },
    Inventory(Box<[Stack]>)
}

impl ItemHolder {
    fn can_recieve(&self, item: &Item, slot_id: usize) -> bool {
        match self {
            ItemHolder::Furnace { fuel: _, material: _, output: _ } => {
                let slot_id = slot_id.into();
                match slot_id {
                    FurnaceSlot::Material => true,
                    FurnaceSlot::Fuel => item == &Item::Coal,
                    FurnaceSlot::Output => false,
                }        
            },
            ItemHolder::Inventory(_) => true,
        }
    }

    pub fn get(&self, slot_id: usize) -> &Stack {
        match self {
            ItemHolder::Furnace { fuel, material, output } => {
                let slot_id = slot_id.into();
                match slot_id {
                    FurnaceSlot::Material => material,
                    FurnaceSlot::Fuel => fuel,
                    FurnaceSlot::Output => output,
                } 
            },
            ItemHolder::Inventory(vec) => &vec[slot_id],
        }
    }

    pub fn get_mut(&mut self, slot_id: usize) -> &mut Stack {
        match self {
            ItemHolder::Furnace { fuel, material, output } => {
                let slot_id = slot_id.into();
                match slot_id {
                    FurnaceSlot::Material => material,
                    FurnaceSlot::Fuel => fuel,
                    FurnaceSlot::Output => output,
                } 
            },
            ItemHolder::Inventory(vec) => &mut vec[slot_id],
        }
    }

    fn take_or_swap(&mut self, stack: &mut Stack, slot_id: usize) {
        let own_stack = self.get_mut(slot_id);
        if own_stack.try_take_from(stack) {
            // we succeeded in taking some items from stack
            return;
        };
        // we weren't able to add anything so we swap
        own_stack.swap_with(stack)
    }

    pub fn try_add(&mut self, stack: Stack) -> Option<Stack> {
        match self {
            ItemHolder::Furnace { fuel, material, output } => todo!(),
            ItemHolder::Inventory(items) => items.try_add(stack),
        }
    }
}

pub fn furnace_slots() -> ItemHolder {
    ItemHolder::Furnace { fuel: Stack::None, material: Stack::None, output: Stack::None }
}

fn item_slot_click(
    mut interaction_query: Query<(&Interaction, &UISlot), Changed<Interaction>>,
    mut dragging: ResMut<Dragging>,
    mut item_holders: Query<&mut ItemHolder>,
) {
    for (interaction, UISlot(item_holder_entt, slot_id)) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Ok(mut clicked_item_holder) = item_holders.get_mut(*item_holder_entt) else {
            continue;
        };
        match &mut dragging.0 {
            Some((dragged_items, _)) => {
                clicked_item_holder.take_or_swap(dragged_items, *slot_id);
                dragging.0 = None;
            },
            None => {
                if clicked_item_holder.get(*slot_id) == &Stack::None {
                    continue;
                }
                let stack = clicked_item_holder.get_mut(*slot_id);
                dragging.0 = Some((stack.take_all(), *item_holder_entt));
            },
        }
    }
}

fn refresh_slot_items(
    node_query: Query<(&UISlot, &Children, Option<&HotBarSlot>)>,
    mut img_query: Query<&mut UiImage>, 
    mut text_query: Query<&mut Text>,
    tex_map: Res<UiTextureMap>,
    selected_slot: Res<SelectedHotbarSlot>,
    item_query: Query<&ItemHolder, Changed<ItemHolder>>,
) {
    for (UISlot(item_holder_entt, slot), children, hotbar_opt) in node_query.iter() {
        let Ok(item_holder) = item_query.get(*item_holder_entt) else {
            continue;
        };
        let stack = item_holder.get(*slot);
        let selected = hotbar_opt.is_some() && *slot == selected_slot.0;
        for child in children {
            if let Ok(mut ui_img) = img_query.get_mut(*child) {
                *ui_img = tex_map.get_with_alpha(stack, if selected { 1. } else { 0.5 });
            }
            if let Ok(mut text) = text_query.get_mut(*child) {
                let quantity = stack.quantity();
                text.sections[0].value = if quantity < 2 { String::new() } else { quantity.to_string() };
            }
        }
    }
}

fn setup_dragging_ui(mut commands: Commands) {
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            width: Val::Percent(SLOT_SIZE_PERCENT),
            aspect_ratio: Some(1.),
            display: Display::None,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(DraggingNode)
    .with_children(
        |node| UiTextureMap::make_empty_item_slot(node, true)
    );
}

fn show_dragging_ui(mut dragging_node: Query<&mut Style, With<DraggingNode>>) {
    let Ok(mut style) = dragging_node.get_single_mut() else {
        return;
    };
    style.display = Display::Flex;
}

fn hide_dragging_ui(mut dragging_node: Query<&mut Style, With<DraggingNode>>) {
    let Ok(mut style) = dragging_node.get_single_mut() else {
        return;
    };
    style.display = Display::None;
}

fn refresh_dragging_ui(
    dragging: Res<Dragging>,
    dragging_node_query: Query<&Children, With<DraggingNode>>,
    mut img_query: Query<&mut UiImage>, 
    mut text_query: Query<&mut Text>,
    tex_map: Res<UiTextureMap>,
) {
    if !dragging.is_changed() {
        return;
    }
    let Ok(children) = dragging_node_query.get_single() else {
        return;
    };
    let stack = match &dragging.0 {
        Some((stack, _)) => stack,
        None => &Stack::None,
    };
    for child in children {
        if let Ok(mut ui_img) = img_query.get_mut(*child) {
            *ui_img = tex_map.get_with_alpha(stack, 1.);
        }
        if let Ok(mut text) = text_query.get_mut(*child) {
            let quantity = stack.quantity();
            text.sections[0].value = if quantity < 2 { String::new() } else { quantity.to_string() };
        }
    }
}

fn refresh_dragging_ui_pos(window: Query<&Window>, mut dragging_node_query: Query<&mut Style, With<DraggingNode>>) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let Some(pos) = window.cursor_position() else {
        return;
    };
    let Ok(mut style) = dragging_node_query.get_single_mut() else {
        return;
    };
    style.left = Val::Px(pos.x);
    style.top = Val::Px(pos.y);
}