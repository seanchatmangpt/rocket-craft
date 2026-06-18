import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { EquipmentPanel } from './ib4-equipment';

describe('EquipmentPanel', () => {
  let panel: EquipmentPanel;

  beforeEach(() => {
    document.body.innerHTML = '';
    panel = new EquipmentPanel();
  });

  afterEach(() => {
    document.body.innerHTML = '';
  });

  it('renders the equipment root element', () => {
    const root = document.getElementById('ib4-equipment-panel');
    expect(root).not.toBeNull();
  });

  it('populates slots based on given loadout', () => {
    panel.updateSlot({ slot: 'weapon', itemName: 'Iron Sword', rarity: 'common', stats: { attack: 10 }, gemSlots: 0, gemsEquipped: [] });
    panel.show();
    
    // Test some text is rendered in the weapon slot
    const slots = document.querySelectorAll('.ib4-equip-item-name');
    let weaponFound = false;
    slots.forEach(slot => {
      if (slot.textContent?.includes('Iron Sword')) {
        weaponFound = true;
      }
    });
    expect(weaponFound).toBe(true);
  });
});
