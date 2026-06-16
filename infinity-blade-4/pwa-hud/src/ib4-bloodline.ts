// ib4-bloodline.ts — BloodlineScreen: rebirth & skill progression for Infinity Blade IV PWA

export interface BloodlinePerk {
  id: string;
  name: string;
  description: string;
  cost: number;       // bloodline points
  unlocked: boolean;
  requires?: string;  // prerequisite perk id
}

const DEFAULT_PERKS: BloodlinePerk[] = [
  // Root perks (no requirements)
  {
    id: 'iron-flesh',
    name: 'Iron Flesh',
    description: 'Increase max health by 20.',
    cost: 1,
    unlocked: false,
  },
  {
    id: 'arcane-blood',
    name: 'Arcane Blood',
    description: 'Increase max magic by 15.',
    cost: 1,
    unlocked: false,
  },
  {
    id: 'keen-edge',
    name: 'Keen Edge',
    description: 'Attack speed increased by 10%.',
    cost: 1,
    unlocked: false,
  },

  // Tier 2 — requires root
  {
    id: 'regeneration',
    name: 'Regeneration',
    description: 'Slowly regenerate health between attacks.',
    cost: 2,
    unlocked: false,
    requires: 'iron-flesh',
  },
  {
    id: 'mana-surge',
    name: 'Mana Surge',
    description: 'Magic abilities deal 25% more damage.',
    cost: 2,
    unlocked: false,
    requires: 'arcane-blood',
  },
  {
    id: 'parry-master',
    name: 'Parry Master',
    description: 'Perfect parry window extended by 0.1s.',
    cost: 2,
    unlocked: false,
    requires: 'keen-edge',
  },

  // Tier 3
  {
    id: 'undying',
    name: 'Undying',
    description: 'Once per battle, survive a killing blow with 1 HP.',
    cost: 3,
    unlocked: false,
    requires: 'regeneration',
  },
  {
    id: 'infinity-mark',
    name: 'Infinity Mark',
    description: 'Charged magic attacks leave a mark that explodes on parry.',
    cost: 3,
    unlocked: false,
    requires: 'mana-surge',
  },
  {
    id: 'bloodline-fury',
    name: 'Bloodline Fury',
    description: 'Combo multiplier increases faster after each kill.',
    cost: 3,
    unlocked: false,
    requires: 'parry-master',
  },
];

export class BloodlineScreen {
  private level: number = 1;
  private xp: number = 0;
  private xpToNext: number = 100;
  private bloodlinePoints: number = 0;
  private perks: BloodlinePerk[] = DEFAULT_PERKS.map((p) => ({ ...p }));

  private screenEl: HTMLElement | null = null;
  private xpBarFill: HTMLElement | null = null;
  private xpLabel: HTMLElement | null = null;
  private levelLabel: HTMLElement | null = null;
  private pointsLabel: HTMLElement | null = null;
  private perkTreeEl: HTMLElement | null = null;
  private deathCauseEl: HTMLElement | null = null;

  constructor() {
    this.injectStyles();
    this.buildScreen();
  }

  // --- Lifecycle ---

  show(deathCause: string): void {
    if (!this.screenEl) return;
    this.screenEl.classList.add('active');
    if (this.deathCauseEl) {
      this.deathCauseEl.textContent = deathCause
        ? `Slain by: ${deathCause}`
        : 'Your bloodline carries on...';
    }
    this.refreshUI();
  }

  hide(): void {
    this.screenEl?.classList.remove('active');
  }

  // --- Callbacks ---

  onRebirthPressed(): void {
    window.dispatchEvent(
      new CustomEvent('ib4:rebirth', { detail: { level: this.level, perks: this.perks } })
    );
    this.hide();
  }

  onPerkSelected(perkId: string): void {
    const perk = this.perks.find((p) => p.id === perkId);
    if (!perk) return;
    if (perk.unlocked) return;
    if (perk.requires) {
      const req = this.perks.find((p) => p.id === perk.requires);
      if (!req?.unlocked) {
        this.showMessage('Prerequisite not met!');
        return;
      }
    }
    if (this.bloodlinePoints < perk.cost) {
      this.showMessage('Not enough bloodline points!');
      return;
    }
    perk.unlocked = true;
    this.bloodlinePoints -= perk.cost;
    this.refreshUI();
    window.dispatchEvent(new CustomEvent('ib4:perk-unlocked', { detail: { perkId } }));
  }

  animateXPGain(amount: number): void {
    const steps = 20;
    const increment = amount / steps;
    let step = 0;
    const tick = () => {
      if (step >= steps) return;
      this.xp = Math.min(this.xp + increment, this.xpToNext);
      step++;
      this.updateXPBar();
      if (step < steps) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }

  // --- Public data setters ---

  setLevel(level: number, xp: number, xpToNext: number, points: number): void {
    this.level = level;
    this.xp = xp;
    this.xpToNext = xpToNext;
    this.bloodlinePoints = points;
    this.refreshUI();
  }

  // --- Perk tree rendering ---

  renderPerkTree(): HTMLElement {
    const tree = document.createElement('div');
    tree.className = 'ib4-perk-tree';

    // Group perks by tier based on 'requires' depth
    const tiers = this.groupByTier();

    tiers.forEach((tier, tierIndex) => {
      const row = document.createElement('div');
      row.className = 'ib4-perk-tier';

      tier.forEach((perk) => {
        row.appendChild(this.renderPerkNode(perk, tierIndex));
      });

      tree.appendChild(row);

      // Add connector row between tiers (except after last)
      if (tierIndex < tiers.length - 1) {
        const connector = document.createElement('div');
        connector.className = 'ib4-perk-connector-row';
        for (let i = 0; i < tier.length; i++) {
          const line = document.createElement('div');
          line.className = 'ib4-perk-connector';
          connector.appendChild(line);
        }
        tree.appendChild(connector);
      }
    });

    return tree;
  }

  // --- Private build ---

  private buildScreen(): void {
    const screen = document.createElement('div');
    screen.id = 'ib4-bloodline-screen';
    screen.className = 'ib4-bloodline-screen';
    this.screenEl = screen;

    // Background decorative rune (ornamental)
    const rune = document.createElement('div');
    rune.className = 'ib4-bl-rune';
    rune.textContent = '✦';
    screen.appendChild(rune);

    // Title
    const title = document.createElement('h1');
    title.className = 'ib4-bl-title';
    title.textContent = 'BLOODLINE';
    screen.appendChild(title);

    // Death cause
    const deathCause = document.createElement('div');
    deathCause.className = 'ib4-bl-death-cause';
    this.deathCauseEl = deathCause;
    screen.appendChild(deathCause);

    // Divider
    screen.appendChild(this.makeDivider());

    // Level row
    const levelRow = document.createElement('div');
    levelRow.className = 'ib4-bl-level-row';

    const levelLabel = document.createElement('div');
    levelLabel.className = 'ib4-bl-level-label';
    levelLabel.textContent = `BLOODLINE LEVEL: I`;
    this.levelLabel = levelLabel;

    const pointsLabel = document.createElement('div');
    pointsLabel.className = 'ib4-bl-points-label';
    pointsLabel.textContent = `${this.bloodlinePoints} pts`;
    this.pointsLabel = pointsLabel;

    levelRow.appendChild(levelLabel);
    levelRow.appendChild(pointsLabel);
    screen.appendChild(levelRow);

    // XP bar
    const xpWrap = document.createElement('div');
    xpWrap.className = 'ib4-bl-xp-wrap';

    const xpTrack = document.createElement('div');
    xpTrack.className = 'ib4-bl-xp-track';

    const xpFill = document.createElement('div');
    xpFill.className = 'ib4-bl-xp-fill';
    xpFill.style.width = '0%';
    this.xpBarFill = xpFill;
    xpTrack.appendChild(xpFill);

    const xpLabel = document.createElement('div');
    xpLabel.className = 'ib4-bl-xp-label';
    xpLabel.textContent = `0 / ${this.xpToNext} XP`;
    this.xpLabel = xpLabel;

    xpWrap.appendChild(xpTrack);
    xpWrap.appendChild(xpLabel);
    screen.appendChild(xpWrap);

    // Divider
    screen.appendChild(this.makeDivider());

    // Perk section title
    const perkTitle = document.createElement('div');
    perkTitle.className = 'ib4-bl-section-title';
    perkTitle.textContent = 'BLOODLINE PERKS';
    screen.appendChild(perkTitle);

    // Perk tree placeholder
    const perkTree = document.createElement('div');
    perkTree.className = 'ib4-perk-tree-wrap';
    perkTree.id = 'ib4-perk-tree';
    this.perkTreeEl = perkTree;
    screen.appendChild(perkTree);

    // Divider
    screen.appendChild(this.makeDivider());

    // Rebirth button
    const rebirthBtn = document.createElement('button');
    rebirthBtn.className = 'ib4-bl-rebirth-btn';
    rebirthBtn.textContent = 'REBIRTH INTO BATTLE';
    rebirthBtn.addEventListener('click', () => this.onRebirthPressed());
    screen.appendChild(rebirthBtn);

    // Message toast (for errors)
    const msg = document.createElement('div');
    msg.className = 'ib4-bl-message';
    msg.id = 'ib4-bl-message';
    screen.appendChild(msg);

    document.body.appendChild(screen);

    // Listen for death event
    window.addEventListener('ib4:rebirth', () => this.hide());
  }

  private renderPerkNode(perk: BloodlinePerk, _tier: number): HTMLElement {
    const prereqMet = !perk.requires ||
      this.perks.find((p) => p.id === perk.requires)?.unlocked === true;
    const canAfford = this.bloodlinePoints >= perk.cost;

    const node = document.createElement('div');
    node.className = 'ib4-perk-node';
    if (perk.unlocked) node.classList.add('unlocked');
    else if (!prereqMet) node.classList.add('locked');
    else if (!canAfford) node.classList.add('cant-afford');
    else node.classList.add('available');

    const nodeIcon = document.createElement('div');
    nodeIcon.className = 'ib4-perk-node-icon';
    nodeIcon.textContent = perk.unlocked ? '✦' : (prereqMet ? '◆' : '◇');

    const nodeName = document.createElement('div');
    nodeName.className = 'ib4-perk-node-name';
    nodeName.textContent = perk.name;

    const nodeDesc = document.createElement('div');
    nodeDesc.className = 'ib4-perk-node-desc';
    nodeDesc.textContent = perk.description;

    const nodeCost = document.createElement('div');
    nodeCost.className = 'ib4-perk-node-cost';
    nodeCost.textContent = perk.unlocked ? 'UNLOCKED' : `${perk.cost} pt${perk.cost !== 1 ? 's' : ''}`;

    node.appendChild(nodeIcon);
    node.appendChild(nodeName);
    node.appendChild(nodeDesc);
    node.appendChild(nodeCost);

    if (!perk.unlocked && prereqMet) {
      node.style.cursor = 'pointer';
      node.addEventListener('click', () => this.onPerkSelected(perk.id));
    }

    return node;
  }

  private groupByTier(): BloodlinePerk[][] {
    const tiers: BloodlinePerk[][] = [];
    // Tier 0: no requires
    tiers.push(this.perks.filter((p) => !p.requires));
    // Tier 1+: requires something in previous tier
    let remaining = this.perks.filter((p) => !!p.requires);
    let prevTier = tiers[0];
    while (remaining.length > 0) {
      const prevIds = new Set(prevTier.map((p) => p.id));
      const currentTier = remaining.filter((p) => p.requires && prevIds.has(p.requires));
      if (currentTier.length === 0) break; // safety
      tiers.push(currentTier);
      remaining = remaining.filter((p) => !currentTier.includes(p));
      prevTier = currentTier;
    }
    return tiers;
  }

  private refreshUI(): void {
    this.updateXPBar();
    if (this.levelLabel) {
      this.levelLabel.textContent = `BLOODLINE LEVEL: ${this.toRoman(this.level)}`;
    }
    if (this.pointsLabel) {
      this.pointsLabel.textContent = `${this.bloodlinePoints} pts`;
    }
    // Re-render perk tree
    if (this.perkTreeEl) {
      this.perkTreeEl.innerHTML = '';
      this.perkTreeEl.appendChild(this.renderPerkTree());
    }
  }

  private updateXPBar(): void {
    if (!this.xpBarFill || !this.xpLabel) return;
    const pct = Math.max(0, Math.min(1, this.xp / this.xpToNext)) * 100;
    this.xpBarFill.style.width = `${pct}%`;
    this.xpLabel.textContent = `${Math.round(this.xp)} / ${this.xpToNext} XP`;
  }

  private showMessage(text: string): void {
    const msg = document.getElementById('ib4-bl-message');
    if (!msg) return;
    msg.textContent = text;
    msg.classList.add('visible');
    setTimeout(() => msg.classList.remove('visible'), 2000);
  }

  private makeDivider(): HTMLElement {
    const d = document.createElement('div');
    d.className = 'ib4-bl-divider';
    return d;
  }

  private toRoman(n: number): string {
    const vals = [1000,900,500,400,100,90,50,40,10,9,5,4,1];
    const syms = ['M','CM','D','CD','C','XC','L','XL','X','IX','V','IV','I'];
    let result = '';
    let num = n;
    for (let i = 0; i < vals.length; i++) {
      while (num >= vals[i]) { result += syms[i]; num -= vals[i]; }
    }
    return result;
  }

  private injectStyles(): void {
    if (document.getElementById('ib4-bloodline-styles')) return;
    const style = document.createElement('style');
    style.id = 'ib4-bloodline-styles';
    style.textContent = `
      .ib4-bloodline-screen {
        position: fixed;
        inset: 0;
        background: radial-gradient(ellipse at center, #1a0800 0%, #000000 70%);
        z-index: 9500;
        display: none;
        flex-direction: column;
        align-items: center;
        padding: 40px 24px 32px;
        box-sizing: border-box;
        overflow-y: auto;
        gap: 18px;
        font-family: 'Palatino Linotype', Palatino, 'Book Antiqua', serif;
        color: #e8d9b0;
      }
      .ib4-bloodline-screen.active { display: flex; }

      .ib4-bl-rune {
        font-size: 5em;
        color: rgba(201,162,39,0.15);
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        pointer-events: none;
        user-select: none;
        font-size: 30vw;
        line-height: 1;
      }

      .ib4-bl-title {
        font-size: 2.8em;
        margin: 0;
        letter-spacing: 8px;
        color: var(--ib-gold, #c9a227);
        text-shadow: 0 0 24px rgba(201,162,39,0.6), 2px 2px 0 rgba(0,0,0,0.7);
        position: relative;
      }
      .ib4-bl-death-cause {
        font-size: 0.9em;
        color: rgba(232,217,176,0.55);
        letter-spacing: 2px;
        font-style: italic;
        position: relative;
      }
      .ib4-bl-divider {
        width: 60%;
        height: 1px;
        background: linear-gradient(90deg, transparent, var(--ib-gold,#c9a227) 30%, var(--ib-gold,#c9a227) 70%, transparent);
        position: relative;
      }
      .ib4-bl-level-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        width: 100%;
        max-width: 580px;
        position: relative;
      }
      .ib4-bl-level-label {
        font-size: 1em;
        letter-spacing: 3px;
        color: var(--ib-gold, #c9a227);
        text-transform: uppercase;
      }
      .ib4-bl-points-label {
        font-size: 1em;
        color: #e8d9b0;
        letter-spacing: 2px;
        font-weight: bold;
      }
      .ib4-bl-xp-wrap {
        width: 100%;
        max-width: 580px;
        display: flex;
        flex-direction: column;
        gap: 5px;
        position: relative;
      }
      .ib4-bl-xp-track {
        width: 100%;
        height: 14px;
        background: rgba(0,0,0,0.6);
        border: 1px solid rgba(201,162,39,0.3);
        border-radius: 7px;
        overflow: hidden;
      }
      .ib4-bl-xp-fill {
        height: 100%;
        background: linear-gradient(90deg, #663300, var(--ib-gold, #c9a227));
        border-radius: 7px;
        transition: width 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
        box-shadow: 0 0 8px rgba(201,162,39,0.5);
      }
      .ib4-bl-xp-label {
        font-size: 0.75em;
        text-align: right;
        color: rgba(232,217,176,0.55);
        letter-spacing: 1px;
      }

      /* Perk section */
      .ib4-bl-section-title {
        font-size: 0.85em;
        letter-spacing: 5px;
        color: rgba(232,217,176,0.55);
        text-transform: uppercase;
        position: relative;
      }
      .ib4-perk-tree-wrap {
        width: 100%;
        max-width: 640px;
        position: relative;
      }
      .ib4-perk-tree {
        display: flex;
        flex-direction: column;
        gap: 0;
        align-items: center;
        width: 100%;
      }
      .ib4-perk-tier {
        display: flex;
        justify-content: center;
        gap: 16px;
        width: 100%;
      }
      .ib4-perk-connector-row {
        display: flex;
        justify-content: center;
        gap: 16px;
        padding: 4px 0;
      }
      .ib4-perk-connector {
        width: 2px;
        height: 20px;
        background: linear-gradient(180deg, rgba(201,162,39,0.3), rgba(201,162,39,0.6));
        flex: 1;
        max-width: 160px;
      }
      .ib4-perk-node {
        flex: 1;
        max-width: 180px;
        min-width: 140px;
        border: 1px solid rgba(201,162,39,0.25);
        border-radius: 6px;
        padding: 10px 12px;
        display: flex;
        flex-direction: column;
        gap: 4px;
        background: rgba(13,10,7,0.8);
        transition: border-color 0.2s, box-shadow 0.2s, background 0.2s;
      }
      .ib4-perk-node.unlocked {
        border-color: var(--ib-gold, #c9a227);
        background: rgba(30,20,5,0.9);
        box-shadow: 0 0 12px rgba(201,162,39,0.25);
      }
      .ib4-perk-node.available {
        border-color: rgba(201,162,39,0.5);
        cursor: pointer;
      }
      .ib4-perk-node.available:hover {
        border-color: var(--ib-gold, #c9a227);
        box-shadow: 0 0 16px rgba(201,162,39,0.35);
        background: rgba(30,20,5,0.95);
      }
      .ib4-perk-node.locked {
        border-color: rgba(100,80,60,0.3);
        opacity: 0.45;
      }
      .ib4-perk-node.cant-afford {
        border-color: rgba(139,0,0,0.4);
        opacity: 0.65;
      }
      .ib4-perk-node-icon {
        font-size: 1.4em;
        color: var(--ib-gold, #c9a227);
        line-height: 1;
      }
      .ib4-perk-node.locked .ib4-perk-node-icon { color: rgba(100,80,60,0.5); }
      .ib4-perk-node-name {
        font-size: 0.85em;
        font-weight: bold;
        letter-spacing: 1px;
        color: #e8d9b0;
      }
      .ib4-perk-node.unlocked .ib4-perk-node-name { color: var(--ib-gold, #c9a227); }
      .ib4-perk-node-desc {
        font-size: 0.7em;
        color: rgba(232,217,176,0.6);
        line-height: 1.4;
      }
      .ib4-perk-node-cost {
        font-size: 0.7em;
        letter-spacing: 1px;
        color: rgba(201,162,39,0.7);
        margin-top: 2px;
      }
      .ib4-perk-node.unlocked .ib4-perk-node-cost { color: #4caf50; }
      .ib4-perk-node.cant-afford .ib4-perk-node-cost { color: rgba(139,0,0,0.8); }

      /* Rebirth button */
      .ib4-bl-rebirth-btn {
        margin-top: 8px;
        padding: 16px 60px;
        background: transparent;
        border: 2px solid var(--ib-gold, #c9a227);
        color: var(--ib-gold, #c9a227);
        font-family: inherit;
        font-size: 1.1em;
        letter-spacing: 4px;
        text-transform: uppercase;
        cursor: pointer;
        border-radius: 4px;
        transition: background 0.25s, box-shadow 0.25s, transform 0.15s;
        position: relative;
      }
      .ib4-bl-rebirth-btn:hover {
        background: rgba(201,162,39,0.12);
        box-shadow: 0 0 28px rgba(201,162,39,0.5);
        transform: scale(1.03);
      }
      .ib4-bl-rebirth-btn:active { transform: scale(0.98); }

      /* Message toast */
      .ib4-bl-message {
        position: fixed;
        bottom: 40px;
        left: 50%;
        transform: translateX(-50%) translateY(20px);
        background: rgba(20,10,5,0.95);
        border: 1px solid rgba(139,0,0,0.6);
        color: #cc4444;
        padding: 10px 28px;
        border-radius: 4px;
        font-size: 0.9em;
        letter-spacing: 2px;
        opacity: 0;
        transition: opacity 0.25s, transform 0.25s;
        pointer-events: none;
      }
      .ib4-bl-message.visible {
        opacity: 1;
        transform: translateX(-50%) translateY(0);
      }
    `;
    document.head.appendChild(style);
  }
}
