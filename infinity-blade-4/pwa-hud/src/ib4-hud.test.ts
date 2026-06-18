import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { InfinityBladeHud } from './ib4-hud';

describe('InfinityBladeHud', () => {
  let hud: InfinityBladeHud;

  beforeEach(() => {
    document.body.innerHTML = '';
    hud = new InfinityBladeHud();
    hud.init();
  });

  afterEach(() => {
    document.body.innerHTML = '';
  });

  it('renders the HUD root element', () => {
    const root = document.getElementById('ib4-hud-root');
    expect(root).not.toBeNull();
  });

  it('updates combo count when onComboUpdated is called', () => {
    hud.onComboUpdated(5, 1.5);
    const countEl = document.querySelector('.ib4-combo-count');
    expect(countEl?.textContent).toBe('x5');
  });

  it('updates health display correctly', () => {
    hud.onPlayerHealthChanged(50, 100);
    const healthText = document.querySelector('.ib4-health-orb text:not([letter-spacing])');
    expect(healthText?.textContent).toBe('50');
  });

  it('sets attack flash direction correctly', () => {
    hud.onAttackInput('overhead');
    const flash = document.querySelector('.ib4-attack-flash');
    expect(flash?.classList.contains('overhead')).toBe(true);
  });
});
