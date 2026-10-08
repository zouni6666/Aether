import type { InjectionKey } from 'vue'
import type { Announcement } from '@/api/announcements'

export const openAnnouncementKey: InjectionKey<(announcement: Announcement) => void> = Symbol('open-announcement')
