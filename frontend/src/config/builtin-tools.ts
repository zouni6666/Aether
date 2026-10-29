import { translate } from '@/i18n'
import { Mail, Shield, AlertTriangle, BellRing } from 'lucide-vue-next'
import type { LucideIcon } from 'lucide-vue-next'

export interface BuiltinTool {
  name: string
  description: string
  href: string
  icon: LucideIcon
}

export const BUILTIN_TOOLS: BuiltinTool[] = [
  {
    get name() { return translate('guide.nav.modules.emailConfig') },
    get description() { return translate('ui.configureSmtpEmailTemplatesAndDeliverySettings') },
    href: '/admin/email',
    icon: Mail,
  },
  {
    get name() { return translate('ui.notificationService') },
    get description() { return translate('ui.manageNotificationsTemplatesAndPushServicePolicies') },
    href: '/admin/notification-service',
    icon: BellRing,
  },
  {
    get name() { return translate('ui.ipSecurity') },
    get description() { return translate('ui.manageIpAllowlistsAndBlocklistsToControlSystemAccess') },
    href: '/admin/ip-security',
    icon: Shield,
  },
  {
    get name() { return translate('ui.auditLogs') },
    get description() { return translate('ui.viewSystemOperationLogsAndTrackSecurityEventsAndChange') },
    href: '/admin/audit-logs',
    icon: AlertTriangle,
  },
]

/** href → display name mapping for breadcrumbs */
export const BUILTIN_TOOL_BREADCRUMBS: Record<string, string> = Object.defineProperties({}, Object.fromEntries(
  BUILTIN_TOOLS.map(tool => [tool.href, { enumerable: true, get: () => tool.name }])
))
