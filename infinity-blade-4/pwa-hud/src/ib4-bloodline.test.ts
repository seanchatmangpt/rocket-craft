import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { BloodlineScreen } from './ib4-bloodline';

describe('BloodlineScreen', () => {
  let screen: BloodlineScreen;

  beforeEach(() => {
    document.body.innerHTML = '';
    screen = new BloodlineScreen();
  });

  afterEach(() => {
    document.body.innerHTML = '';
  });

  it('renders the bloodline root element', () => {
    const root = document.getElementById('ib4-bloodline-screen');
    expect(root).not.toBeNull();
  });

  it('updates title when level changes', () => {
    screen.setLevel(5, 0, 100, 0);
    const levelLabel = document.querySelector('.ib4-bl-level-label');
    expect(levelLabel?.textContent).toContain('V');
  });
});
