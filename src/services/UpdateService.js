import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';

// Gitee is used as a mirror/fallback for users with poor GitHub connectivity
const GITHUB_API = 'https://api.github.com/repos/yarnbyte/AI-SecondLife-Talk/releases/latest';
const GITEE_API = 'https://gitee.com/api/v5/repos/yarnbyte/AI-SecondLife-Talk/releases/latest';

export class UpdateService {
  /**
   * Checks for a new version by racing requests to GitHub and Gitee.
   * Returns an object if an update is found, otherwise null.
   */
  static async checkUpdate() {
    try {
      const currentVersion = await getVersion();
      let releaseInfo = null;
      
      try {
        const controller = new AbortController();
        // 5s timeout
        const timeoutId = setTimeout(() => controller.abort(), 5000);
        
        // Race both APIs to get the fastest successful response
        const fetchGithub = fetch(GITHUB_API, { signal: controller.signal }).then(res => {
          if (!res.ok) throw new Error('GitHub HTTP ' + res.status);
          return res.json().then(data => ({ ...data, source: 'github' }));
        });
        
        const fetchGitee = fetch(GITEE_API, { signal: controller.signal }).then(res => {
          if (!res.ok) throw new Error('Gitee HTTP ' + res.status);
          return res.json().then(data => ({ ...data, source: 'gitee' }));
        });
        
        releaseInfo = await Promise.any([fetchGithub, fetchGitee]);
        clearTimeout(timeoutId);
      } catch (e) {
        console.warn('Update check failed (all mirrors timed out or errored):', e);
        return null;
      }
      
      if (!releaseInfo || !releaseInfo.tag_name) return null;
      
      // tag_name is usually "v0.1.0" or "0.1.0"
      const remoteVersion = releaseInfo.tag_name.trim().replace(/^v/, '');
      
      if (this.compareVersion(remoteVersion, currentVersion) > 0) {
         return {
           hasUpdate: true,
           currentVersion,
           remoteVersion,
           source: releaseInfo.source,
           // Redirect to the web page containing the releases
           url: releaseInfo.source === 'gitee' 
             ? 'https://gitee.com/yarnbyte/AI-SecondLife-Talk/releases'
             : 'https://github.com/yarnbyte/AI-SecondLife-Talk/releases/latest'
         };
      }
      return { hasUpdate: false, currentVersion };
    } catch (e) {
      console.warn('Failed to retrieve local version or check update:', e);
      return null;
    }
  }

  static async openUpdateUrl(url) {
    try {
      await openUrl(url);
    } catch (e) {
      console.error('Failed to open update URL:', e);
    }
  }

  // Returns >0 if v1 is newer than v2
  static compareVersion(v1, v2) {
    const parts1 = v1.split('.').map(Number);
    const parts2 = v2.split('.').map(Number);
    for (let i = 0; i < Math.max(parts1.length, parts2.length); i++) {
        const p1 = parts1[i] || 0;
        const p2 = parts2[i] || 0;
        if (p1 > p2) return 1;
        if (p1 < p2) return -1;
    }
    return 0;
  }
}
