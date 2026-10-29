// OAuth 设备授权（Kiro / Windsurf / xAI）的会话状态定义

export type DeviceAuthType = 'default' | 'google' | 'github' | 'builder_id' | 'identity_center' | 'device'

export type WindsurfLoginOption = 'default' | 'google' | 'github'

export interface DeviceAuthState {
  auth_type: DeviceAuthType
  start_url: string
  region: string
  totp_secret: string
  callback_url: string
  callback_required: boolean
  starting: boolean
  completing: boolean
  session_id: string
  user_code: string
  verification_uri: string
  verification_uri_complete: string
  expires_at: number  // unix timestamp (ms)
  interval: number    // 轮询间隔 (秒)
  status: 'idle' | 'pending' | 'authorized' | 'expired' | 'error'
  error: string
}

export function createInitialDeviceState(): DeviceAuthState {
  return {
    auth_type: 'google',
    start_url: '',
    region: 'eu-north-1',
    totp_secret: '',
    callback_url: '',
    callback_required: false,
    starting: false,
    completing: false,
    session_id: '',
    user_code: '',
    verification_uri: '',
    verification_uri_complete: '',
    expires_at: 0,
    interval: 5,
    status: 'idle',
    error: '',
  }
}
