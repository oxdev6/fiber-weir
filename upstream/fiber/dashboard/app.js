const toast = document.querySelector('#toast');
let toastTimer;

document.querySelector('#streamModal')?.remove();
document.querySelector('#failureButton')?.remove();
document.querySelector('.stream-panel')?.replaceChildren(Object.assign(document.createElement('div'), {
  className: 'empty-state',
  innerHTML: '<span class="empty-icon">◇</span><h2>No connected stream</h2><p>Connect a Fiber node to create and monitor conditional payment streams.</p><button class="button primary" type="button" data-connect>Connect Fiber node</button>'
}));
document.querySelector('.condition-panel')?.replaceChildren(Object.assign(document.createElement('div'), {
  className: 'empty-state',
  innerHTML: '<span class="empty-icon">⌁</span><h2>Predicate verifier unavailable</h2><p>Live script, proof, confirmation, and commitment data will appear here after the node connection is established.</p>'
}));
document.querySelector('.events-panel')?.replaceChildren(Object.assign(document.createElement('div'), {
  className: 'empty-state compact',
  innerHTML: '<span class="empty-icon">◷</span><h2>No settlement events</h2><p>There is no connected stream history to display.</p>'
}));
document.querySelector('.contract-panel')?.replaceChildren(Object.assign(document.createElement('div'), {
  className: 'empty-state compact',
  innerHTML: '<span class="empty-icon">◇</span><h2>Contract status unavailable</h2><p>Connect the node and configured CKB contract deployment to inspect enforcement status.</p>'
}));
async function connectFiberNode() {
  const buttons = document.querySelectorAll('[data-connect], #connectButton');
  buttons.forEach((button) => { button.disabled = true; button.classList.add('loading'); });
  try {
    const response = await fetch('http://127.0.0.1:8227', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'node_info', params: [] })
    });
    if (!response.ok) throw new Error(`RPC returned ${response.status}`);
    const payload = await response.json();
    if (payload.error) throw new Error(payload.error.message || 'RPC request failed');
    const result = payload.result || {};
    const channelResponse = await fetch('http://127.0.0.1:8227', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jsonrpc: '2.0', id: 2, method: 'list_channels', params: [{}] })
    });
    const channelPayload = await channelResponse.json();
    const channelCount = channelPayload.result?.channels?.length;
    document.querySelector('#streamStatus').textContent = 'CONNECTED · READY';
    document.querySelector('#streamMessage').textContent = 'Fiber node RPC is reachable. Conditional stream RPC is not installed in this node build.';
    document.querySelector('#workspaceName').textContent = 'Fiber node connected';
    document.querySelector('#workspaceMessage').textContent = 'RPC 127.0.0.1:8227';
    document.querySelector('#nodeStatusText').textContent = 'Node connected';
    document.querySelector('#nodeId').textContent = result.node_id || result.pubkey || 'RPC connection established';
    document.querySelector('#nodeMeta').textContent = `${channelCount ?? 0} channel(s) · JSON-RPC online`;
    document.querySelector('#syncText').textContent = 'RPC connected';
    document.querySelector('#syncTime').textContent = 'just now';
    ['#workspaceStatus', '#nodeStatus'].forEach((selector) => document.querySelector(selector)?.classList.replace('offline', 'online'));
    showToast('Fiber node connected; conditional stream API unavailable');
  } catch (error) {
    const message = error instanceof TypeError
      ? 'Fiber RPC is unreachable or CORS is not enabled for this dashboard origin'
      : `Unable to connect to Fiber RPC: ${error.message}`;
    showToast(message);
  } finally {
    buttons.forEach((button) => { button.disabled = false; button.classList.remove('loading'); });
  }
}

document.querySelectorAll('[data-connect], #connectButton').forEach((button) => button.addEventListener('click', connectFiberNode));

function showToast(message) {
  toast.textContent = message;
  toast.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toast.classList.remove('show'), 3000);
}

const accountButton = document.querySelector('#accountButton');
const accountMenu = document.querySelector('#accountMenu');

accountButton?.addEventListener('click', (event) => {
  event.stopPropagation();
  const open = accountMenu.hidden;
  accountMenu.hidden = !open;
  accountButton.setAttribute('aria-expanded', String(open));
});

document.querySelectorAll('[data-account-action]').forEach((action) => action.addEventListener('click', () => {
  const messages = {
    settings: 'Account settings will be available after node authentication is connected',
    switch: 'Operator switching will be available after authentication is connected',
    logout: 'You are not signed in to a Fiber node'
  };
  accountMenu.hidden = true;
  accountButton?.setAttribute('aria-expanded', 'false');
  showToast(messages[action.dataset.accountAction]);
}));

document.addEventListener('keydown', (event) => {
  if (event.key === 'Escape' && accountMenu && !accountMenu.hidden) {
    accountMenu.hidden = true;
    accountButton?.setAttribute('aria-expanded', 'false');
    accountButton?.focus();
  }
});

document.addEventListener('click', (event) => {
  if (accountMenu && !accountMenu.hidden && !accountMenu.contains(event.target) && event.target !== accountButton) {
    accountMenu.hidden = true;
    accountButton?.setAttribute('aria-expanded', 'false');
  }
});

document.querySelectorAll('.copy-button').forEach((button) => button.addEventListener('click', async () => {
  try {
    await navigator.clipboard.writeText(button.dataset.copy);
    showToast('Value copied');
  } catch {
    showToast('Copy is unavailable in this browser context');
  }
}));

document.querySelectorAll('.nav-item').forEach((item) => item.addEventListener('click', () => {
  document.querySelector('.nav-item.active')?.classList.remove('active');
  item.classList.add('active');
  document.querySelector('#sidebar')?.classList.remove('open');
}));

document.querySelector('#mobileMenu')?.addEventListener('click', () => document.querySelector('#sidebar')?.classList.toggle('open'));
document.addEventListener('click', (event) => {
  const sidebar = document.querySelector('#sidebar');
  const mobileMenu = document.querySelector('#mobileMenu');
  if (window.innerWidth <= 700 && sidebar?.classList.contains('open') && !sidebar.contains(event.target) && event.target !== mobileMenu) {
    sidebar.classList.remove('open');
  }
});
