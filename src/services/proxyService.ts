import type { ProxySettings } from '../types/countdown';

export const normalizeProxyPort = (port: string) => {
  const normalized = port.trim();
  if (!/^\d{1,5}$/.test(normalized)) {
    return null;
  }

  const portNumber = Number(normalized);
  if (!Number.isInteger(portNumber) || portNumber < 1 || portNumber > 65535) {
    return null;
  }

  return String(portNumber);
};

export const getActiveProxyPort = (settings: ProxySettings) => {
  if (!settings.enabled) {
    return null;
  }

  return normalizeProxyPort(settings.port);
};

export const getProxyPortNumber = (settings: ProxySettings) => {
  const port = getActiveProxyPort(settings);
  return port ? Number(port) : null;
};