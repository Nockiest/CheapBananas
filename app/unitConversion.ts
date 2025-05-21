// cheap_bananas/app/unitConversion.ts

export const UNIT_CONVERSIONS: Record<string, { to: string; factor: number }> = {
  g: { to: 'kg', factor: 0.001 },
  kg: { to: 'kg', factor: 1 },
  mg: { to: 'kg', factor: 0.000001 },
  l: { to: 'l', factor: 1 },
  ml: { to: 'l', factor: 0.001 },
  hl: { to: 'l', factor: 100 },
  ks: { to: 'ks', factor: 1 },
};

export function normalizeUnit(value: string, unit: string) {
  const num = parseFloat(value);
  if (isNaN(num) || !unit) return { value, unit };
  const conv = UNIT_CONVERSIONS[unit as keyof typeof UNIT_CONVERSIONS];
  if (!conv) return { value, unit };
  return { value: (num * conv.factor).toString(), unit: conv.to };
}

export function normalizePricePerPiece(price: string, volume: string) {
  const p = parseFloat(price);
  const v = parseFloat(volume);
  if (isNaN(p) || isNaN(v) || v === 0) return price;
  return (p / v).toString();
}
