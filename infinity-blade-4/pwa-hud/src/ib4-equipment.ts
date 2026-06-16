// ib4-equipment.ts — EquipmentPanel: slide-in equipment display for Infinity Blade IV PWA

export interface EquipmentSlot {
  slot: 'weapon' | 'shield' | 'helmet' | 'armor' | 'ring';
  itemName: string;
  rarity: 'common' | 'uncommon' | 'rare' | 'epic' | 'infinity';
  stats: { attack?: number; defense?: number; magic?: number };
  gemSlots: number;
  gemsEquipped: string[];
}

const SLOT_ICONS: Record<string, string> = {
  weapon:  '⚔',
  shield:  '🛡',
  helmet:  '⛑',
  armor:   '🥋',
  ring:    '💍',
};

const RARITY_COLORS: Record<string, string> = {
  common:   '#9e9e9e',
  uncommon: '#4caf50',
  rare:     '#2196f3',
  epic:     '#9c27b0',
  infinity: '#c9a227',
};

const RARITY_LABELS: Record<string, string> = {
  common:   'Common',
  uncommon: 'Uncommon',
  rare:     'Rare',
  epic:     'Epic',
  infinity: 'Infinity',
};

export class EquipmentPanel {
  private slots: Map<string, EquipmentSlot> = new Map();
  private panelEl: HTMLElement | null = null;
  private visible: boolean = false;

  constructor() {
    this.injectStyles();
    this.buildPanel();
    this.populateDefaults();
  }

  // --- Lifecycle ---

  show(): void {
    if (!this.panelEl) return;
    this.visible = true;
    this.panelEl.classList.add('open');
    this.renderAllSlots();
  }

  hide(): void {
    if (!this.panelEl) return;
    this.visible = false;
    this.panelEl.classList.remove('open');
  }

  toggle(): void {
    this.visible ? this.hide() : this.show();
  }

  // --- Data ---

  updateSlot(slot: EquipmentSlot): void {
    this.slots.set(slot.slot, slot);
    if (this.visible) {
      this.refreshSlotCard(slot);
    }
  }

  // --- Event callbacks ---

  onUpgradePressed(slotName: string): void {
    window.dispatchEvent(
      new CustomEvent('ib4:upgrade-equipment', { detail: { slot: slotName } })
    );
  }

  // --- Rendering ---

  renderSlotCard(slot: EquipmentSlot): HTMLElement {
    const card = document.createElement('div');
    card.className = 'ib4-equip-card';
    card.setAttribute('data-slot', slot.slot);
    const rarityColor = this.getRarityColor(slot.rarity);
    card.style.borderColor = rarityColor;
    card.style.boxShadow = `0 0 10px ${rarityColor}44, inset 0 0 8px rgba(0,0,0,0.5)`;

    // Header row: icon + name
    const header = document.createElement('div');
    header.className = 'ib4-equip-card-header';

    const icon = document.createElement('span');
    icon.className = 'ib4-equip-slot-icon';
    icon.textContent = SLOT_ICONS[slot.slot] || '?';

    const nameWrap = document.createElement('div');
    nameWrap.className = 'ib4-equip-name-wrap';

    const slotLabel = document.createElement('div');
    slotLabel.className = 'ib4-equip-slot-label';
    slotLabel.textContent = slot.slot.toUpperCase();

    const itemName = document.createElement('div');
    itemName.className = 'ib4-equip-item-name';
    itemName.textContent = slot.itemName;
    itemName.style.color = rarityColor;

    const rarityBadge = document.createElement('span');
    rarityBadge.className = 'ib4-rarity-badge';
    rarityBadge.textContent = RARITY_LABELS[slot.rarity];
    rarityBadge.style.color = rarityColor;
    rarityBadge.style.borderColor = `${rarityColor}88`;

    nameWrap.appendChild(slotLabel);
    nameWrap.appendChild(itemName);
    nameWrap.appendChild(rarityBadge);

    header.appendChild(icon);
    header.appendChild(nameWrap);
    card.appendChild(header);

    // Stats row
    const statsRow = document.createElement('div');
    statsRow.className = 'ib4-equip-stats';
    if (slot.stats.attack !== undefined) {
      statsRow.appendChild(this.statChip('ATK', slot.stats.attack, '#cc4400'));
    }
    if (slot.stats.defense !== undefined) {
      statsRow.appendChild(this.statChip('DEF', slot.stats.defense, '#2266aa'));
    }
    if (slot.stats.magic !== undefined) {
      statsRow.appendChild(this.statChip('MAG', slot.stats.magic, '#6622aa'));
    }
    card.appendChild(statsRow);

    // Gem slots
    if (slot.gemSlots > 0) {
      const gemRow = document.createElement('div');
      gemRow.className = 'ib4-gem-row';
      for (let i = 0; i < slot.gemSlots; i++) {
        const gem = document.createElement('span');
        gem.className = 'ib4-gem-slot';
        const equipped = slot.gemsEquipped[i];
        if (equipped) {
          gem.textContent = '◆';
          gem.title = equipped;
          gem.classList.add('filled');
        } else {
          gem.textContent = '◇';
          gem.classList.add('empty');
        }
        gemRow.appendChild(gem);
      }
      card.appendChild(gemRow);
    }

    // Upgrade button
    const upgradeBtn = document.createElement('button');
    upgradeBtn.className = 'ib4-upgrade-btn';
    upgradeBtn.textContent = 'UPGRADE';
    upgradeBtn.addEventListener('click', () => this.onUpgradePressed(slot.slot));
    card.appendChild(upgradeBtn);

    return card;
  }

  getRarityColor(rarity: string): string {
    return RARITY_COLORS[rarity] ?? RARITY_COLORS.common;
  }

  // --- Private helpers ---

  private statChip(label: string, value: number, color: string): HTMLElement {
    const chip = document.createElement('div');
    chip.className = 'ib4-stat-chip';
    chip.style.borderColor = `${color}66`;
    const lbl = document.createElement('span');
    lbl.className = 'ib4-stat-chip-label';
    lbl.textContent = label;
    lbl.style.color = color;
    const val = document.createElement('span');
    val.className = 'ib4-stat-chip-val';
    val.textContent = String(value);
    chip.appendChild(lbl);
    chip.appendChild(val);
    return chip;
  }

  private buildPanel(): void {
    const panel = document.createElement('div');
    panel.id = 'ib4-equipment-panel';
    panel.className = 'ib4-equipment-panel';

    // Header
    const header = document.createElement('div');
    header.className = 'ib4-equip-panel-header';

    const title = document.createElement('h2');
    title.className = 'ib4-equip-panel-title';
    title.textContent = 'EQUIPMENT';

    const closeBtn = document.createElement('button');
    closeBtn.className = 'ib4-equip-close-btn';
    closeBtn.textContent = '×';
    closeBtn.addEventListener('click', () => this.hide());

    header.appendChild(title);
    header.appendChild(closeBtn);
    panel.appendChild(header);

    // Divider
    const divider = document.createElement('div');
    divider.className = 'ib4-equip-divider';
    panel.appendChild(divider);

    // Cards container
    const cardsWrap = document.createElement('div');
    cardsWrap.className = 'ib4-equip-cards';
    cardsWrap.id = 'ib4-equip-cards';
    panel.appendChild(cardsWrap);

    document.body.appendChild(panel);
    this.panelEl = panel;

    // Listen for external open event
    window.addEventListener('ib4:open-equipment', () => this.show());
  }

  private populateDefaults(): void {
    const defaults: EquipmentSlot[] = [
      {
        slot: 'weapon', itemName: 'Iron Sword', rarity: 'common',
        stats: { attack: 12 }, gemSlots: 1, gemsEquipped: [],
      },
      {
        slot: 'shield', itemName: 'Wooden Buckler', rarity: 'common',
        stats: { defense: 8 }, gemSlots: 1, gemsEquipped: [],
      },
      {
        slot: 'helmet', itemName: 'Iron Cap', rarity: 'common',
        stats: { defense: 5 }, gemSlots: 1, gemsEquipped: [],
      },
      {
        slot: 'armor', itemName: 'Chain Mail', rarity: 'uncommon',
        stats: { defense: 15, magic: 3 }, gemSlots: 2, gemsEquipped: ['Ruby'],
      },
      {
        slot: 'ring', itemName: 'Ring of Flame', rarity: 'rare',
        stats: { magic: 20 }, gemSlots: 2, gemsEquipped: ['Sapphire', 'Emerald'],
      },
    ];
    defaults.forEach((s) => this.slots.set(s.slot, s));
  }

  private renderAllSlots(): void {
    const cardsWrap = document.getElementById('ib4-equip-cards');
    if (!cardsWrap) return;
    cardsWrap.innerHTML = '';
    const order: Array<EquipmentSlot['slot']> = ['weapon', 'shield', 'helmet', 'armor', 'ring'];
    order.forEach((slotName) => {
      const slot = this.slots.get(slotName);
      if (slot) {
        cardsWrap.appendChild(this.renderSlotCard(slot));
      }
    });
  }

  private refreshSlotCard(slot: EquipmentSlot): void {
    const cardsWrap = document.getElementById('ib4-equip-cards');
    if (!cardsWrap) return;
    const existing = cardsWrap.querySelector(`[data-slot="${slot.slot}"]`);
    const newCard = this.renderSlotCard(slot);
    if (existing) {
      cardsWrap.replaceChild(newCard, existing);
    } else {
      cardsWrap.appendChild(newCard);
    }
  }

  private injectStyles(): void {
    if (document.getElementById('ib4-equipment-styles')) return;
    const style = document.createElement('style');
    style.id = 'ib4-equipment-styles';
    style.textContent = `
      .ib4-equipment-panel {
        position: fixed;
        top: 0;
        right: -420px;
        width: 400px;
        max-width: 100vw;
        height: 100vh;
        background: rgba(10, 7, 4, 0.97);
        border-left: 2px solid var(--ib-gold, #c9a227);
        box-shadow: -8px 0 32px rgba(0,0,0,0.9);
        z-index: 9000;
        overflow-y: auto;
        padding: 24px;
        box-sizing: border-box;
        transition: right 0.35s cubic-bezier(0.25, 0.8, 0.25, 1);
        font-family: 'Palatino Linotype', Palatino, 'Book Antiqua', serif;
        color: #e8d9b0;
        display: flex;
        flex-direction: column;
        gap: 16px;
      }
      .ib4-equipment-panel.open {
        right: 0;
      }
      .ib4-equip-panel-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
      }
      .ib4-equip-panel-title {
        margin: 0;
        font-size: 1.5em;
        letter-spacing: 5px;
        color: var(--ib-gold, #c9a227);
        text-shadow: 0 0 8px rgba(201,162,39,0.5);
      }
      .ib4-equip-close-btn {
        background: none;
        border: none;
        color: #9e9e9e;
        font-size: 2em;
        cursor: pointer;
        line-height: 1;
        padding: 0;
        transition: color 0.2s;
      }
      .ib4-equip-close-btn:hover { color: #e8d9b0; }
      .ib4-equip-divider {
        height: 1px;
        background: linear-gradient(90deg, transparent, var(--ib-gold, #c9a227), transparent);
        margin: 0 -4px;
      }
      .ib4-equip-cards {
        display: flex;
        flex-direction: column;
        gap: 14px;
      }
      .ib4-equip-card {
        border: 1px solid #9e9e9e;
        border-radius: 6px;
        padding: 14px;
        background: rgba(20, 14, 8, 0.8);
        display: flex;
        flex-direction: column;
        gap: 10px;
        transition: box-shadow 0.2s;
      }
      .ib4-equip-card-header {
        display: flex;
        align-items: flex-start;
        gap: 12px;
      }
      .ib4-equip-slot-icon {
        font-size: 2em;
        line-height: 1;
        flex-shrink: 0;
        margin-top: 2px;
      }
      .ib4-equip-name-wrap {
        display: flex;
        flex-direction: column;
        gap: 2px;
        flex: 1;
        min-width: 0;
      }
      .ib4-equip-slot-label {
        font-size: 0.65em;
        letter-spacing: 3px;
        color: rgba(232,217,176,0.5);
      }
      .ib4-equip-item-name {
        font-size: 1.05em;
        font-weight: bold;
        letter-spacing: 1px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }
      .ib4-rarity-badge {
        display: inline-block;
        font-size: 0.65em;
        letter-spacing: 2px;
        border: 1px solid;
        border-radius: 3px;
        padding: 1px 6px;
        margin-top: 2px;
      }
      .ib4-equip-stats {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
      }
      .ib4-stat-chip {
        display: flex;
        align-items: center;
        gap: 4px;
        border: 1px solid;
        border-radius: 4px;
        padding: 3px 8px;
        background: rgba(0,0,0,0.4);
        font-size: 0.8em;
      }
      .ib4-stat-chip-label {
        font-weight: bold;
        letter-spacing: 1px;
      }
      .ib4-stat-chip-val {
        color: #e8d9b0;
        font-weight: bold;
      }
      .ib4-gem-row {
        display: flex;
        gap: 6px;
        align-items: center;
      }
      .ib4-gem-slot {
        font-size: 1.2em;
        cursor: default;
      }
      .ib4-gem-slot.filled { color: #c9a227; }
      .ib4-gem-slot.empty  { color: rgba(232,217,176,0.3); }
      .ib4-upgrade-btn {
        background: transparent;
        border: 1px solid rgba(201,162,39,0.5);
        color: #c9a227;
        font-family: inherit;
        font-size: 0.8em;
        letter-spacing: 3px;
        padding: 7px 16px;
        cursor: pointer;
        border-radius: 4px;
        transition: background 0.2s, box-shadow 0.2s;
        align-self: flex-start;
      }
      .ib4-upgrade-btn:hover {
        background: rgba(201,162,39,0.12);
        box-shadow: 0 0 10px rgba(201,162,39,0.3);
      }
    `;
    document.head.appendChild(style);
  }
}
