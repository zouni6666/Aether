import { translate } from '@/i18n'
const regionLabels: Record<string, string> = {
  get US() { return translate('ui.usUnitedStates') },
  get SG() { return translate('ui.sgSingapore') },
  get JP() { return translate('ui.jpJapan') },
  get KR() { return translate('ui.krSouthKorea') },
  get HK() { return translate('ui.hkHongKongChina') },
  get TW() { return translate('ui.twTaiwanChina') },
  get DE() { return translate('ui.deGermany') },
  get GB() { return translate('ui.gbUnitedKingdom') },
  get FR() { return translate('ui.frFrance') },
  get NL() { return translate('ui.nlNetherlands') },
  get AU() { return translate('ui.auAustralia') },
  get CA() { return translate('ui.caCanada') },
  get IN() { return translate('ui.inIndia') },
  get BR() { return translate('ui.brBrazil') },
  get RU() { return translate('ui.ruRussia') },
  get CN() { return translate('ui.cnChina') },
}

export function formatRegion(region: string | null, fallback = '-'): string {
  if (!region) return fallback
  return regionLabels[region.toUpperCase()] || region
}
