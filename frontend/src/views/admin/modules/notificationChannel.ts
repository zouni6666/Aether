import { translate } from '@/i18n'
import { getI18nLocale } from '@/i18n'

const CHANNEL_LABELS_ZH: Record<string, string> = {
  bark: 'Bark',
  get server_chan() { return translate('ui.serverChan') },
  get email() { return translate('ui.email') },
  get user_email() { return translate('ui.userEmail') },
  get module() { return translate('ui.module') },
  get item() { return translate('ui.notificationItems') },
  get none() { return translate('ui.noServicesAvailable') },
}

const CHANNEL_LABELS_EN: Record<string, string> = {
  bark: 'Bark',
  server_chan: 'ServerChan',
  email: 'Email',
  user_email: 'User email',
  module: 'Module',
  item: 'Notification item',
  none: 'No service available',
}

export function formatNotificationChannel(channel: string): string {
  const labels = getI18nLocale() === 'en-US' ? CHANNEL_LABELS_EN : CHANNEL_LABELS_ZH
  return labels[channel] ?? channel
}
