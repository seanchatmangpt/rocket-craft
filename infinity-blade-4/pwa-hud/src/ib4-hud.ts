// ib4-hud.ts — InfinityBladeHud: combat HUD for Infinity Blade IV PWA
// Mirrors pwa-staff/src/hud.ts architecture: DOM manipulation, event-driven updates

export class InfinityBladeHud {
  // --- State ---
  private playerHealth: number = 100;
  private playerMaxHealth: number = 100;
  private playerMagic: number = 50;
  private playerMaxMagic: number = 50;
  private enemyHealth: number = 100;
  private enemyMaxHealth: number = 100;
  private comboCount: number = 0;
  private comboMultiplier: number = 1.0;
  private bloodlineLevel: number = 1;
  private currentXP: number = 0;
  private xpToNext: number = 100;
  private equippedWeapon: string = 'Iron Sword';
  private magicType: 'fire' | 'lightning' | 'ice' = 'fire';
  private isParryWindowActive: boolean = false;
  private isPerfectParryActive: boolean = false;

  // --- Root container ---
  private hudRoot: HTMLElement | null = null;

  // --- DOM references ---
  private healthOrbSvg: SVGElement | null = null;
  private healthOrbPath: SVGPathElement | null = null;
  private healthOrbText: SVGTextElement | null = null;
  private healthOrbGlow: SVGCircleElement | null = null;
  private magicBarFill: HTMLElement | null = null;
  private magicBarLabel: HTMLElement | null = null;
  private enemyHpBar: HTMLElement | null = null;
  private enemyHpFill: HTMLElement | null = null;
  private enemyHpLabel: HTMLElement | null = null;
  private comboCounter: HTMLElement | null = null;
  private comboMultiplierEl: HTMLElement | null = null;
  private bloodlineBadge: HTMLElement | null = null;
  private xpBarFill: HTMLElement | null = null;
  private xpBarLabel: HTMLElement | null = null;
  private parryRingOverlay: HTMLElement | null = null;
  private perfectParryOverlay: HTMLElement | null = null;
  private equipmentBtn: HTMLElement | null = null;
  private weaponLabel: HTMLElement | null = null;
  private magicIcon: HTMLElement | null = null;
  private attackFlash: HTMLElement | null = null;

  // Touch tracking
  private touchStart: Touch | null = null;

  // --- Init ---

  init(): void {
    this.injectStyles();
    this.buildHUD();
    this.setupTouchControls();
    this.render();
  }

  show(): void {
    if (this.hudRoot) {
      this.hudRoot.style.display = 'block';
    }
  }

  hide(): void {
    if (this.hudRoot) {
      this.hudRoot.style.display = 'none';
    }
  }

  // --- Style injection ---

  private injectStyles(): void {
    if (document.getElementById('ib4-hud-styles')) return;
    const style = document.createElement('style');
    style.id = 'ib4-hud-styles';
    style.textContent = `
      :root {
        --ib-gold: #c9a227;
        --ib-gold-bright: #f0c040;
        --ib-blood: #8b0000;
        --ib-blood-bright: #cc1111;
        --ib-magic: #1a3a5c;
        --ib-magic-fire: #ff6600;
        --ib-magic-lightning: #4488ff;
        --ib-magic-ice: #00cccc;
        --ib-dark: #0d0a07;
        --ib-panel: rgba(13, 10, 7, 0.85);
        --ib-border: rgba(201, 162, 39, 0.4);
        --ib-text: #e8d9b0;
        --ib-text-dim: rgba(232, 217, 176, 0.55);
        --ib-common: #9e9e9e;
        --ib-uncommon: #4caf50;
        --ib-rare: #2196f3;
        --ib-epic: #9c27b0;
        --ib-infinity: #c9a227;
      }

      #ib4-hud-root {
        position: fixed;
        inset: 0;
        pointer-events: none;
        z-index: 8000;
        font-family: 'Palatino Linotype', Palatino, 'Book Antiqua', serif;
        user-select: none;
      }

      /* ---- HEALTH ORB (bottom-left) ---- */
      .ib4-health-orb {
        position: absolute;
        bottom: 28px;
        left: 28px;
        width: 110px;
        height: 110px;
        pointer-events: auto;
        filter: drop-shadow(0 0 12px rgba(139, 0, 0, 0.6));
        transition: filter 0.3s ease;
      }
      .ib4-health-orb.low-health {
        animation: ib4-orb-pulse 0.8s ease-in-out infinite alternate;
        filter: drop-shadow(0 0 22px rgba(255, 20, 20, 0.9));
      }
      @keyframes ib4-orb-pulse {
        from { filter: drop-shadow(0 0 12px rgba(255, 20, 20, 0.7)); }
        to   { filter: drop-shadow(0 0 28px rgba(255, 80, 0, 1)); }
      }

      /* ---- MAGIC BAR (bottom-center) ---- */
      .ib4-magic-bar-wrap {
        position: absolute;
        bottom: 28px;
        left: 50%;
        transform: translateX(-50%);
        width: 240px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 4px;
        pointer-events: auto;
      }
      .ib4-magic-bar-label {
        font-size: 0.7em;
        letter-spacing: 2px;
        text-transform: uppercase;
        color: var(--ib-text-dim);
        display: flex;
        align-items: center;
        gap: 6px;
      }
      .ib4-magic-icon {
        font-size: 1.1em;
      }
      .ib4-magic-bar-track {
        width: 100%;
        height: 12px;
        background: rgba(0,0,0,0.6);
        border: 1px solid var(--ib-border);
        border-radius: 6px;
        overflow: hidden;
        box-shadow: inset 0 1px 4px rgba(0,0,0,0.8), 0 0 8px rgba(26, 58, 92, 0.5);
      }
      .ib4-magic-bar-fill {
        height: 100%;
        border-radius: 6px;
        transition: width 0.4s cubic-bezier(0.25, 0.8, 0.25, 1), background-color 0.4s ease;
        box-shadow: 0 0 10px currentColor;
      }
      .ib4-magic-bar-fill.fire {
        background: linear-gradient(90deg, #991100, var(--ib-magic-fire));
        color: var(--ib-magic-fire);
      }
      .ib4-magic-bar-fill.lightning {
        background: linear-gradient(90deg, #112266, var(--ib-magic-lightning));
        color: var(--ib-magic-lightning);
      }
      .ib4-magic-bar-fill.ice {
        background: linear-gradient(90deg, #006666, var(--ib-magic-ice));
        color: var(--ib-magic-ice);
      }

      /* ---- EQUIPMENT BUTTON (bottom-center-right offset) ---- */
      .ib4-equip-btn {
        position: absolute;
        bottom: 28px;
        right: 160px;
        width: 52px;
        height: 52px;
        border-radius: 50%;
        background: var(--ib-panel);
        border: 2px solid var(--ib-gold);
        color: var(--ib-gold);
        font-size: 1.4em;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        pointer-events: auto;
        box-shadow: 0 0 10px rgba(201, 162, 39, 0.3);
        transition: box-shadow 0.2s, transform 0.2s;
      }
      .ib4-equip-btn:hover, .ib4-equip-btn:active {
        box-shadow: 0 0 20px rgba(201, 162, 39, 0.7);
        transform: scale(1.1);
      }

      /* ---- WEAPON LABEL ---- */
      .ib4-weapon-label {
        position: absolute;
        bottom: 50px;
        left: 150px;
        color: var(--ib-text-dim);
        font-size: 0.75em;
        letter-spacing: 1px;
        font-style: italic;
        text-shadow: 0 0 6px rgba(201, 162, 39, 0.4);
      }

      /* ---- ENEMY HP BAR (top-center) ---- */
      .ib4-enemy-hp {
        position: absolute;
        top: 24px;
        left: 50%;
        transform: translateX(-50%);
        width: 360px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 5px;
      }
      .ib4-enemy-hp-label {
        font-size: 0.9em;
        letter-spacing: 3px;
        text-transform: uppercase;
        color: var(--ib-gold);
        text-shadow: 0 0 8px rgba(201, 162, 39, 0.6);
      }
      .ib4-enemy-hp-track {
        width: 100%;
        height: 16px;
        background: rgba(0,0,0,0.7);
        border: 1px solid rgba(139, 0, 0, 0.5);
        border-radius: 3px;
        overflow: hidden;
        box-shadow: 0 0 12px rgba(139, 0, 0, 0.4), inset 0 1px 4px rgba(0,0,0,0.8);
      }
      .ib4-enemy-hp-fill {
        height: 100%;
        background: linear-gradient(90deg, var(--ib-blood), var(--ib-blood-bright));
        border-radius: 3px;
        transition: width 0.5s cubic-bezier(0.25, 0.8, 0.25, 1);
        box-shadow: 0 0 8px rgba(204, 17, 17, 0.6);
      }

      /* ---- COMBO COUNTER (top-right) ---- */
      .ib4-combo {
        position: absolute;
        top: 20px;
        right: 24px;
        text-align: right;
        line-height: 1;
      }
      .ib4-combo-count {
        font-size: 3.5em;
        font-weight: bold;
        color: var(--ib-gold);
        text-shadow: 0 0 16px rgba(201, 162, 39, 0.7), 2px 2px 0 rgba(0,0,0,0.8);
        transition: transform 0.15s cubic-bezier(0.34, 1.56, 0.64, 1);
        display: block;
        letter-spacing: -1px;
      }
      .ib4-combo-count.bump {
        transform: scale(1.35);
      }
      .ib4-combo-multiplier {
        font-size: 1em;
        color: var(--ib-text);
        letter-spacing: 2px;
        text-shadow: 0 0 6px rgba(201, 162, 39, 0.4);
      }

      /* ---- BLOODLINE BADGE (bottom-right) ---- */
      .ib4-bloodline {
        position: absolute;
        bottom: 28px;
        right: 28px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 4px;
        pointer-events: auto;
      }
      .ib4-xp-bar-track {
        width: 80px;
        height: 5px;
        background: rgba(0,0,0,0.6);
        border: 1px solid var(--ib-border);
        border-radius: 3px;
        overflow: hidden;
      }
      .ib4-xp-bar-fill {
        height: 100%;
        background: linear-gradient(90deg, #664400, var(--ib-gold));
        border-radius: 3px;
        transition: width 0.6s cubic-bezier(0.25, 0.8, 0.25, 1);
        box-shadow: 0 0 6px rgba(201, 162, 39, 0.5);
      }
      .ib4-xp-label {
        font-size: 0.6em;
        color: var(--ib-text-dim);
        letter-spacing: 1px;
      }
      .ib4-bloodline-badge {
        width: 80px;
        height: 80px;
        border-radius: 50%;
        border: 2px solid var(--ib-gold);
        background: radial-gradient(circle at 40% 35%, #2a1800, #0d0a07);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.5em;
        color: var(--ib-gold);
        text-shadow: 0 0 12px rgba(201, 162, 39, 0.8);
        box-shadow: 0 0 14px rgba(201, 162, 39, 0.35), inset 0 0 10px rgba(0,0,0,0.7);
        letter-spacing: 1px;
      }
      .ib4-bloodline-text {
        font-size: 0.65em;
        color: var(--ib-text-dim);
        letter-spacing: 2px;
        text-transform: uppercase;
      }

      /* ---- PARRY RING OVERLAY (center-screen) ---- */
      .ib4-parry-ring {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%) scale(0.6);
        width: 180px;
        height: 180px;
        border-radius: 50%;
        border: 4px solid var(--ib-gold);
        opacity: 0;
        transition: opacity 0.15s ease, transform 0.15s ease;
        box-shadow: 0 0 30px rgba(201, 162, 39, 0.5), inset 0 0 20px rgba(201, 162, 39, 0.15);
        pointer-events: none;
      }
      .ib4-parry-ring.active {
        opacity: 1;
        transform: translate(-50%, -50%) scale(1);
        animation: ib4-parry-pulse 0.5s ease-in-out infinite alternate;
      }
      @keyframes ib4-parry-pulse {
        from { box-shadow: 0 0 20px rgba(201, 162, 39, 0.4), inset 0 0 10px rgba(201, 162, 39, 0.1); }
        to   { box-shadow: 0 0 50px rgba(201, 162, 39, 0.9), inset 0 0 30px rgba(201, 162, 39, 0.3); }
      }

      /* ---- PERFECT PARRY OVERLAY ---- */
      .ib4-perfect-parry {
        position: absolute;
        inset: 0;
        background: rgba(201, 162, 39, 0);
        display: flex;
        align-items: center;
        justify-content: center;
        pointer-events: none;
        opacity: 0;
        transition: opacity 0.1s ease;
      }
      .ib4-perfect-parry.active {
        animation: ib4-perfect-flash 0.9s ease forwards;
      }
      @keyframes ib4-perfect-flash {
        0%   { opacity: 0; background: rgba(201, 162, 39, 0); }
        10%  { opacity: 1; background: rgba(201, 162, 39, 0.55); }
        25%  { opacity: 1; background: rgba(201, 162, 39, 0.35); }
        80%  { opacity: 1; background: rgba(201, 162, 39, 0.05); }
        100% { opacity: 0; background: rgba(201, 162, 39, 0); }
      }
      .ib4-perfect-parry-text {
        font-size: 3.2em;
        font-weight: bold;
        color: #fff;
        text-shadow:
          0 0 20px rgba(201, 162, 39, 1),
          0 0 40px rgba(255, 200, 0, 0.8),
          3px 3px 0 rgba(0,0,0,0.6);
        letter-spacing: 4px;
        text-transform: uppercase;
        animation: ib4-perfect-text-pop 0.9s ease forwards;
      }
      @keyframes ib4-perfect-text-pop {
        0%   { transform: scale(0.5); opacity: 0; }
        15%  { transform: scale(1.3); opacity: 1; }
        30%  { transform: scale(1.0); }
        80%  { transform: scale(1.05); opacity: 1; }
        100% { transform: scale(1.1); opacity: 0; }
      }

      /* ---- ATTACK FLASH ---- */
      .ib4-attack-flash {
        position: absolute;
        inset: 0;
        pointer-events: none;
        opacity: 0;
        background: transparent;
        transition: opacity 0.05s ease;
      }
      .ib4-attack-flash.overhead {
        background: linear-gradient(180deg, rgba(255,255,100,0.3) 0%, transparent 50%);
      }
      .ib4-attack-flash.left {
        background: linear-gradient(90deg, rgba(200,100,255,0.3) 0%, transparent 50%);
      }
      .ib4-attack-flash.right {
        background: linear-gradient(270deg, rgba(200,100,255,0.3) 0%, transparent 50%);
      }
      .ib4-attack-flash.flash-in {
        opacity: 1;
      }

      /* ---- TITAN DEFEATED OVERLAY ---- */
      .ib4-victory-overlay {
        position: absolute;
        inset: 0;
        background: radial-gradient(circle at center, rgba(201, 162, 39, 0.18) 0%, rgba(0,0,0,0.85) 70%);
        display: none;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 20px;
        pointer-events: auto;
        z-index: 100;
      }
      .ib4-victory-overlay.active { display: flex; }
      .ib4-victory-title {
        font-size: 2.8em;
        color: var(--ib-gold);
        text-shadow: 0 0 30px rgba(201, 162, 39, 0.8), 2px 2px 0 rgba(0,0,0,0.7);
        letter-spacing: 5px;
        text-transform: uppercase;
        animation: ib4-victory-appear 0.6s cubic-bezier(0.34, 1.56, 0.64, 1) both;
      }
      @keyframes ib4-victory-appear {
        from { transform: scale(0.3); opacity: 0; }
        to   { transform: scale(1); opacity: 1; }
      }
      .ib4-victory-titan {
        font-size: 1.3em;
        color: var(--ib-text);
        letter-spacing: 2px;
      }
      .ib4-victory-loot {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        color: var(--ib-text-dim);
        font-size: 0.9em;
        letter-spacing: 1px;
      }
      .ib4-victory-loot-item {
        color: var(--ib-gold-bright);
        padding: 4px 16px;
        border: 1px solid var(--ib-border);
        border-radius: 4px;
        background: rgba(0,0,0,0.5);
      }
      .ib4-victory-continue-btn {
        margin-top: 20px;
        padding: 12px 40px;
        background: var(--ib-panel);
        border: 2px solid var(--ib-gold);
        color: var(--ib-gold);
        font-family: inherit;
        font-size: 1em;
        letter-spacing: 3px;
        text-transform: uppercase;
        cursor: pointer;
        border-radius: 4px;
        transition: background 0.2s, box-shadow 0.2s;
      }
      .ib4-victory-continue-btn:hover {
        background: rgba(201, 162, 39, 0.15);
        box-shadow: 0 0 20px rgba(201, 162, 39, 0.5);
      }

      /* ---- DEATH OVERLAY ---- */
      .ib4-death-overlay {
        position: absolute;
        inset: 0;
        background: rgba(60, 0, 0, 0.85);
        display: none;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 24px;
        pointer-events: auto;
        z-index: 100;
      }
      .ib4-death-overlay.active { display: flex; }
      .ib4-death-title {
        font-size: 3em;
        color: var(--ib-blood-bright);
        text-shadow: 0 0 30px rgba(204, 17, 17, 0.9), 2px 2px 0 rgba(0,0,0,0.7);
        letter-spacing: 6px;
        text-transform: uppercase;
      }
      .ib4-death-subtitle {
        color: var(--ib-text-dim);
        font-size: 1em;
        letter-spacing: 2px;
        font-style: italic;
      }
      .ib4-rebirth-btn {
        padding: 14px 50px;
        background: var(--ib-panel);
        border: 2px solid var(--ib-blood-bright);
        color: var(--ib-blood-bright);
        font-family: inherit;
        font-size: 1.1em;
        letter-spacing: 3px;
        text-transform: uppercase;
        cursor: pointer;
        border-radius: 4px;
        transition: background 0.2s, box-shadow 0.2s;
      }
      .ib4-rebirth-btn:hover {
        background: rgba(139, 0, 0, 0.25);
        box-shadow: 0 0 20px rgba(204, 17, 17, 0.6);
      }
    `;
    document.head.appendChild(style);
  }

  // --- HUD DOM construction ---

  private buildHUD(): void {
    const root = document.createElement('div');
    root.id = 'ib4-hud-root';
    this.hudRoot = root;

    // Health Orb (bottom-left)
    root.appendChild(this.buildHealthOrb());

    // Magic Bar (bottom-center)
    root.appendChild(this.buildMagicBar());

    // Weapon label
    const weaponLabel = document.createElement('div');
    weaponLabel.className = 'ib4-weapon-label';
    weaponLabel.textContent = this.equippedWeapon;
    this.weaponLabel = weaponLabel;
    root.appendChild(weaponLabel);

    // Enemy HP Bar (top-center)
    root.appendChild(this.buildEnemyHpBar());

    // Combo counter (top-right)
    root.appendChild(this.buildComboCounter());

    // Bloodline + XP (bottom-right)
    root.appendChild(this.buildBloodlineBadge());

    // Equipment button
    const equipBtn = document.createElement('div');
    equipBtn.className = 'ib4-equip-btn';
    equipBtn.textContent = '⚔';
    equipBtn.title = 'Equipment';
    equipBtn.addEventListener('click', () => {
      window.dispatchEvent(new CustomEvent('ib4:open-equipment'));
    });
    this.equipmentBtn = equipBtn;
    root.appendChild(equipBtn);

    // Parry ring overlay
    const parryRing = document.createElement('div');
    parryRing.className = 'ib4-parry-ring';
    this.parryRingOverlay = parryRing;
    root.appendChild(parryRing);

    // Perfect parry overlay
    root.appendChild(this.buildPerfectParryOverlay());

    // Attack flash overlay
    const attackFlash = document.createElement('div');
    attackFlash.className = 'ib4-attack-flash';
    this.attackFlash = attackFlash;
    root.appendChild(attackFlash);

    // Victory overlay (hidden by default)
    const victoryOverlay = document.createElement('div');
    victoryOverlay.className = 'ib4-victory-overlay';
    victoryOverlay.id = 'ib4-victory-overlay';
    root.appendChild(victoryOverlay);

    // Death overlay (hidden by default)
    const deathOverlay = document.createElement('div');
    deathOverlay.className = 'ib4-death-overlay';
    deathOverlay.id = 'ib4-death-overlay';
    root.appendChild(deathOverlay);

    document.body.appendChild(root);
  }

  private buildHealthOrb(): SVGElement {
    const ns = 'http://www.w3.org/2000/svg';
    const svg = document.createElementNS(ns, 'svg') as unknown as SVGElement;
    svg.setAttribute('class', 'ib4-health-orb');
    svg.setAttribute('viewBox', '0 0 110 110');

    // Outer ring
    const ring = document.createElementNS(ns, 'circle');
    ring.setAttribute('cx', '55');
    ring.setAttribute('cy', '55');
    ring.setAttribute('r', '50');
    ring.setAttribute('fill', 'none');
    ring.setAttribute('stroke', 'rgba(201,162,39,0.6)');
    ring.setAttribute('stroke-width', '3');
    svg.appendChild(ring);

    // Background fill
    const bg = document.createElementNS(ns, 'circle');
    bg.setAttribute('cx', '55');
    bg.setAttribute('cy', '55');
    bg.setAttribute('r', '47');
    bg.setAttribute('fill', '#0d0a07');
    svg.appendChild(bg);

    // Glow circle (controlled to indicate low health)
    const glow = document.createElementNS(ns, 'circle') as unknown as SVGCircleElement;
    glow.setAttribute('cx', '55');
    glow.setAttribute('cy', '55');
    glow.setAttribute('r', '47');
    glow.setAttribute('fill', 'rgba(139,0,0,0.2)');
    this.healthOrbGlow = glow;
    svg.appendChild(glow);

    // Health fill path — liquid fill using clip trick
    const defs = document.createElementNS(ns, 'defs');
    const clipPath = document.createElementNS(ns, 'clipPath');
    clipPath.setAttribute('id', 'ib4-health-clip');
    const clipCircle = document.createElementNS(ns, 'circle');
    clipCircle.setAttribute('cx', '55');
    clipCircle.setAttribute('cy', '55');
    clipCircle.setAttribute('r', '47');
    clipPath.appendChild(clipCircle);
    defs.appendChild(clipPath);
    svg.appendChild(defs);

    const fillRect = document.createElementNS(ns, 'rect') as unknown as SVGPathElement;
    fillRect.setAttribute('x', '8');
    fillRect.setAttribute('y', '8');
    fillRect.setAttribute('width', '94');
    fillRect.setAttribute('height', '94');
    fillRect.setAttribute('fill', 'url(#ib4-hp-grad)');
    fillRect.setAttribute('clip-path', 'url(#ib4-health-clip)');
    this.healthOrbPath = fillRect;
    svg.appendChild(fillRect);

    const gradDef = document.createElementNS(ns, 'defs');
    const grad = document.createElementNS(ns, 'linearGradient');
    grad.setAttribute('id', 'ib4-hp-grad');
    grad.setAttribute('x1', '0');
    grad.setAttribute('y1', '0');
    grad.setAttribute('x2', '0');
    grad.setAttribute('y2', '1');
    const stop1 = document.createElementNS(ns, 'stop');
    stop1.setAttribute('offset', '0%');
    stop1.setAttribute('stop-color', '#cc1111');
    const stop2 = document.createElementNS(ns, 'stop');
    stop2.setAttribute('offset', '100%');
    stop2.setAttribute('stop-color', '#8b0000');
    grad.appendChild(stop1);
    grad.appendChild(stop2);
    gradDef.appendChild(grad);
    svg.appendChild(gradDef);

    // HP text
    const hpText = document.createElementNS(ns, 'text') as unknown as SVGTextElement;
    hpText.setAttribute('x', '55');
    hpText.setAttribute('y', '60');
    hpText.setAttribute('text-anchor', 'middle');
    hpText.setAttribute('dominant-baseline', 'middle');
    hpText.setAttribute('fill', '#e8d9b0');
    hpText.setAttribute('font-size', '20');
    hpText.setAttribute('font-family', 'Palatino Linotype, serif');
    hpText.setAttribute('font-weight', 'bold');
    hpText.textContent = '100';
    this.healthOrbText = hpText;
    svg.appendChild(hpText);

    // "HP" label
    const hpLabel = document.createElementNS(ns, 'text');
    hpLabel.setAttribute('x', '55');
    hpLabel.setAttribute('y', '78');
    hpLabel.setAttribute('text-anchor', 'middle');
    hpLabel.setAttribute('fill', 'rgba(232,217,176,0.5)');
    hpLabel.setAttribute('font-size', '10');
    hpLabel.setAttribute('font-family', 'Palatino Linotype, serif');
    hpLabel.setAttribute('letter-spacing', '2');
    hpLabel.textContent = 'HP';
    svg.appendChild(hpLabel);

    return svg;
  }

  private buildMagicBar(): HTMLElement {
    const wrap = document.createElement('div');
    wrap.className = 'ib4-magic-bar-wrap';

    const label = document.createElement('div');
    label.className = 'ib4-magic-bar-label';

    const icon = document.createElement('span');
    icon.className = 'ib4-magic-icon';
    icon.textContent = this.getMagicIcon(this.magicType);
    this.magicIcon = icon;

    const labelText = document.createElement('span');
    labelText.textContent = 'MAGIC';
    this.magicBarLabel = labelText;

    label.appendChild(icon);
    label.appendChild(labelText);

    const track = document.createElement('div');
    track.className = 'ib4-magic-bar-track';

    const fill = document.createElement('div');
    fill.className = `ib4-magic-bar-fill ${this.magicType}`;
    fill.style.width = '100%';
    this.magicBarFill = fill;

    track.appendChild(fill);
    wrap.appendChild(label);
    wrap.appendChild(track);
    return wrap;
  }

  private buildEnemyHpBar(): HTMLElement {
    const wrap = document.createElement('div');
    wrap.className = 'ib4-enemy-hp';
    this.enemyHpBar = wrap;

    const label = document.createElement('div');
    label.className = 'ib4-enemy-hp-label';
    label.textContent = 'TITAN';
    this.enemyHpLabel = label;

    const track = document.createElement('div');
    track.className = 'ib4-enemy-hp-track';

    const fill = document.createElement('div');
    fill.className = 'ib4-enemy-hp-fill';
    fill.style.width = '100%';
    this.enemyHpFill = fill;

    track.appendChild(fill);
    wrap.appendChild(label);
    wrap.appendChild(track);
    return wrap;
  }

  private buildComboCounter(): HTMLElement {
    const wrap = document.createElement('div');
    wrap.className = 'ib4-combo';

    const count = document.createElement('span');
    count.className = 'ib4-combo-count';
    count.textContent = 'x0';
    this.comboCounter = count;

    const multiplier = document.createElement('div');
    multiplier.className = 'ib4-combo-multiplier';
    multiplier.textContent = 'x1.0';
    this.comboMultiplierEl = multiplier;

    wrap.appendChild(count);
    wrap.appendChild(multiplier);
    return wrap;
  }

  private buildBloodlineBadge(): HTMLElement {
    const wrap = document.createElement('div');
    wrap.className = 'ib4-bloodline';

    const xpTrack = document.createElement('div');
    xpTrack.className = 'ib4-xp-bar-track';

    const xpFill = document.createElement('div');
    xpFill.className = 'ib4-xp-bar-fill';
    xpFill.style.width = '0%';
    this.xpBarFill = xpFill;
    xpTrack.appendChild(xpFill);

    const xpLabel = document.createElement('div');
    xpLabel.className = 'ib4-xp-label';
    xpLabel.textContent = '0 / 100 XP';
    this.xpBarLabel = xpLabel;

    const badge = document.createElement('div');
    badge.className = 'ib4-bloodline-badge';
    badge.textContent = this.toRoman(this.bloodlineLevel);
    this.bloodlineBadge = badge;

    const text = document.createElement('div');
    text.className = 'ib4-bloodline-text';
    text.textContent = 'BLOODLINE';

    wrap.appendChild(xpTrack);
    wrap.appendChild(xpLabel);
    wrap.appendChild(badge);
    wrap.appendChild(text);
    return wrap;
  }

  private buildPerfectParryOverlay(): HTMLElement {
    const overlay = document.createElement('div');
    overlay.className = 'ib4-perfect-parry';
    this.perfectParryOverlay = overlay;

    const text = document.createElement('div');
    text.className = 'ib4-perfect-parry-text';
    text.textContent = 'PERFECT PARRY!';
    overlay.appendChild(text);
    return overlay;
  }

  // --- Render state into DOM ---

  private render(): void {
    this.renderHealth();
    this.renderMagic();
    this.renderEnemyHp();
    this.renderCombo();
    this.renderBloodline();
  }

  private renderHealth(): void {
    if (!this.healthOrbPath || !this.healthOrbText || !this.healthOrbGlow) return;
    const pct = Math.max(0, Math.min(1, this.playerHealth / this.playerMaxHealth));
    // Clip the fill rect by adjusting y offset to simulate liquid fill
    const fillY = 8 + (1 - pct) * 94;
    this.healthOrbPath.setAttribute('y', String(fillY));
    this.healthOrbPath.setAttribute('height', String(pct * 94));
    this.healthOrbText.textContent = String(Math.round(this.playerHealth));

    const orbEl = this.hudRoot?.querySelector('.ib4-health-orb');
    if (pct <= 0.25) {
      orbEl?.classList.add('low-health');
      this.healthOrbGlow.setAttribute('fill', 'rgba(255,20,20,0.3)');
    } else {
      orbEl?.classList.remove('low-health');
      this.healthOrbGlow.setAttribute('fill', 'rgba(139,0,0,0.2)');
    }
  }

  private renderMagic(): void {
    if (!this.magicBarFill) return;
    const pct = Math.max(0, Math.min(1, this.playerMagic / this.playerMaxMagic)) * 100;
    this.magicBarFill.style.width = `${pct}%`;
  }

  private renderEnemyHp(): void {
    if (!this.enemyHpFill) return;
    const pct = Math.max(0, Math.min(1, this.enemyHealth / this.enemyMaxHealth)) * 100;
    this.enemyHpFill.style.width = `${pct}%`;
  }

  private renderCombo(): void {
    if (!this.comboCounter || !this.comboMultiplierEl) return;
    this.comboCounter.textContent = `x${this.comboCount}`;
    this.comboMultiplierEl.textContent = `x${this.comboMultiplier.toFixed(1)}`;
  }

  private renderBloodline(): void {
    if (!this.xpBarFill || !this.xpBarLabel || !this.bloodlineBadge) return;
    const pct = Math.max(0, Math.min(1, this.currentXP / this.xpToNext)) * 100;
    this.xpBarFill.style.width = `${pct}%`;
    this.xpBarLabel.textContent = `${this.currentXP} / ${this.xpToNext} XP`;
    this.bloodlineBadge.textContent = this.toRoman(this.bloodlineLevel);
  }

  // --- Public update methods ---

  onPlayerHealthChanged(health: number, maxHealth: number): void {
    this.playerHealth = health;
    this.playerMaxHealth = maxHealth;
    this.renderHealth();
  }

  onPlayerMagicChanged(magic: number, maxMagic: number): void {
    this.playerMagic = magic;
    this.playerMaxMagic = maxMagic;
    this.renderMagic();
  }

  onEnemyHealthChanged(health: number): void {
    this.enemyHealth = health;
    this.renderEnemyHp();
  }

  onComboUpdated(count: number, multiplier: number): void {
    this.comboCount = count;
    this.comboMultiplier = multiplier;
    this.renderCombo();

    // Bump animation
    if (this.comboCounter && count > 0) {
      this.comboCounter.classList.remove('bump');
      // Trigger reflow to restart animation
      void this.comboCounter.offsetWidth;
      this.comboCounter.classList.add('bump');
      setTimeout(() => this.comboCounter?.classList.remove('bump'), 200);
    }
  }

  onBloodlineLevelUp(level: number, xp: number): void {
    this.bloodlineLevel = level;
    this.currentXP = xp;
    this.renderBloodline();

    if (this.bloodlineBadge) {
      this.bloodlineBadge.style.animation = 'none';
      void this.bloodlineBadge.offsetWidth;
      this.bloodlineBadge.style.animation =
        'ib4-victory-appear 0.6s cubic-bezier(0.34,1.56,0.64,1) both';
    }
  }

  onXPGained(xp: number, total: number, max: number): void {
    this.currentXP = total;
    this.xpToNext = max;
    this.renderBloodline();
  }

  onParryWindowOpen(): void {
    this.isParryWindowActive = true;
    this.parryRingOverlay?.classList.add('active');
  }

  onParryWindowClose(): void {
    this.isParryWindowActive = false;
    this.parryRingOverlay?.classList.remove('active');
  }

  onPerfectParry(): void {
    this.isPerfectParryActive = true;
    this.onParryWindowClose();

    if (this.perfectParryOverlay) {
      this.perfectParryOverlay.classList.remove('active');
      void this.perfectParryOverlay.offsetWidth;
      this.perfectParryOverlay.classList.add('active');
      setTimeout(() => {
        this.perfectParryOverlay?.classList.remove('active');
        this.isPerfectParryActive = false;
      }, 950);
    }
  }

  onComboBreak(): void {
    if (this.comboCounter) {
      this.comboCounter.style.color = 'var(--ib-blood-bright)';
      setTimeout(() => {
        if (this.comboCounter) this.comboCounter.style.color = 'var(--ib-gold)';
      }, 400);
    }
    this.comboCount = 0;
    this.comboMultiplier = 1.0;
    this.renderCombo();
  }

  onTitanDefeated(titanName: string, loot: string[]): void {
    const overlay = this.hudRoot?.querySelector('#ib4-victory-overlay') as HTMLElement | null;
    if (!overlay) return;

    overlay.innerHTML = '';
    overlay.classList.add('active');

    const title = document.createElement('div');
    title.className = 'ib4-victory-title';
    title.textContent = 'TITAN SLAIN';

    const titanLabel = document.createElement('div');
    titanLabel.className = 'ib4-victory-titan';
    titanLabel.textContent = titanName;

    const lootWrap = document.createElement('div');
    lootWrap.className = 'ib4-victory-loot';
    const lootTitle = document.createElement('div');
    lootTitle.textContent = 'Loot Acquired:';
    lootWrap.appendChild(lootTitle);
    loot.forEach((item) => {
      const li = document.createElement('div');
      li.className = 'ib4-victory-loot-item';
      li.textContent = item;
      lootWrap.appendChild(li);
    });

    const continueBtn = document.createElement('button');
    continueBtn.className = 'ib4-victory-continue-btn';
    continueBtn.textContent = 'CONTINUE';
    continueBtn.addEventListener('click', () => {
      overlay.classList.remove('active');
      window.dispatchEvent(new CustomEvent('ib4:continue-after-victory'));
    });

    overlay.appendChild(title);
    overlay.appendChild(titanLabel);
    overlay.appendChild(lootWrap);
    overlay.appendChild(continueBtn);
  }

  onDeath(): void {
    const overlay = this.hudRoot?.querySelector('#ib4-death-overlay') as HTMLElement | null;
    if (!overlay) return;

    overlay.innerHTML = '';
    overlay.classList.add('active');

    const title = document.createElement('div');
    title.className = 'ib4-death-title';
    title.textContent = 'YOU HAVE FALLEN';

    const subtitle = document.createElement('div');
    subtitle.className = 'ib4-death-subtitle';
    subtitle.textContent = 'Your bloodline carries on...';

    const rebirthBtn = document.createElement('button');
    rebirthBtn.className = 'ib4-rebirth-btn';
    rebirthBtn.textContent = 'REBIRTH';
    rebirthBtn.addEventListener('click', () => {
      overlay.classList.remove('active');
      window.dispatchEvent(new CustomEvent('ib4:rebirth'));
    });

    overlay.appendChild(title);
    overlay.appendChild(subtitle);
    overlay.appendChild(rebirthBtn);
  }

  onEquipmentChanged(slot: string, itemName: string): void {
    if (slot === 'weapon') {
      this.equippedWeapon = itemName;
      if (this.weaponLabel) {
        this.weaponLabel.textContent = itemName;
      }
    }
    window.dispatchEvent(new CustomEvent('ib4:equipment-changed', { detail: { slot, itemName } }));
  }

  onMagicTypeChanged(type: 'fire' | 'lightning' | 'ice'): void {
    this.magicType = type;
    if (this.magicBarFill) {
      this.magicBarFill.className = `ib4-magic-bar-fill ${type}`;
    }
    if (this.magicIcon) {
      this.magicIcon.textContent = this.getMagicIcon(type);
    }
  }

  onAttackInput(direction: 'overhead' | 'left' | 'right'): void {
    if (!this.attackFlash) return;
    this.attackFlash.className = `ib4-attack-flash ${direction} flash-in`;
    setTimeout(() => {
      if (this.attackFlash) {
        this.attackFlash.classList.remove('flash-in');
      }
    }, 120);
  }

  setEnemyName(name: string): void {
    if (this.enemyHpLabel) {
      this.enemyHpLabel.textContent = name.toUpperCase();
    }
  }

  // --- Touch / Swipe controls ---

  private setupTouchControls(): void {
    document.addEventListener('touchstart', (e: TouchEvent) => {
      this.touchStart = e.changedTouches[0];
    }, { passive: true });

    document.addEventListener('touchend', (e: TouchEvent) => {
      if (!this.touchStart) return;
      const touchEnd = e.changedTouches[0];
      const direction = this.detectSwipeDirection(this.touchStart, touchEnd);
      this.touchStart = null;
      if (direction === 'dodge') {
        window.dispatchEvent(new CustomEvent('ib4:dodge'));
      } else if (direction) {
        this.onAttackInput(direction);
        window.dispatchEvent(new CustomEvent('ib4:attack', { detail: { direction } }));
      }
    }, { passive: true });
  }

  private detectSwipeDirection(
    start: Touch,
    end: Touch
  ): 'overhead' | 'left' | 'right' | 'dodge' | null {
    const dx = end.clientX - start.clientX;
    const dy = end.clientY - start.clientY;
    const dist = Math.sqrt(dx * dx + dy * dy);

    if (dist < 40) return null; // Too short — tap, not swipe

    const angle = Math.atan2(dy, dx) * (180 / Math.PI);

    // Down swipe = dodge
    if (angle > 60 && angle < 120) return 'dodge';
    // Up swipe = overhead
    if (angle < -60 && angle > -120) return 'overhead';
    // Right swipe
    if (Math.abs(angle) <= 60 && dx > 0) return 'right';
    // Left swipe
    if (Math.abs(angle) <= 60 && dx < 0) return 'left';
    // Diagonal up-right / up-left = overhead
    if (angle < -30 && angle > -150) return 'overhead';

    return null;
  }

  // --- Utility ---

  private getMagicIcon(type: 'fire' | 'lightning' | 'ice'): string {
    switch (type) {
      case 'fire': return '🔥';
      case 'lightning': return '⚡';
      case 'ice': return '❄';
    }
  }

  private toRoman(n: number): string {
    const vals = [1000,900,500,400,100,90,50,40,10,9,5,4,1];
    const syms = ['M','CM','D','CD','C','XC','L','XL','X','IX','V','IV','I'];
    let result = '';
    let num = n;
    for (let i = 0; i < vals.length; i++) {
      while (num >= vals[i]) {
        result += syms[i];
        num -= vals[i];
      }
    }
    return result;
  }
}

// Auto-init if loaded directly
if (typeof window !== 'undefined') {
  const hud = new InfinityBladeHud();
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => hud.init());
  } else {
    hud.init();
  }
  (window as any).ib4Hud = hud;
}
