let currentUserId = null;
let currentUsername = '';
let currentChatUserId = null;
let ws = null;
let chatModal = null;
let mediaRecorder = null;
let recordedChunks = [];
let chatRefreshInterval;



// Add new refresh functions
function startChatRefresh() {
    // Initial load
    refreshCurrentChat();
    // Start interval
    chatRefreshInterval = setInterval(refreshCurrentChat, 100);
}

async function refreshCurrentChat() {
    if (currentChatUserId) {
        try {
            const response = await fetch(`/messages/${currentChatUserId}`);
            const data = await response.json();
            if (data.status === 'success') {
                const messagesContainer = document.getElementById('messages');
                messagesContainer.innerHTML = '';
                data.messages.forEach(message => displayMessage(message));
                messagesContainer.scrollTop = messagesContainer.scrollHeight;
            }
        } catch (error) {
            console.error('Failed to refresh messages:', error);
        }
    }
}


document.addEventListener('DOMContentLoaded', () => {
    chatModal = new bootstrap.Modal(document.getElementById('newChatModal'));
    initializeEventListeners();
});

function initializeEventListeners() {
    // Authentication listeners
    document.getElementById('show-register').addEventListener('click', showRegisterPage);
    document.getElementById('show-login').addEventListener('click', showLoginPage);
    document.getElementById('login-form').addEventListener('submit', handleLogin);
    document.getElementById('register-form').addEventListener('submit', handleRegister);

    // Chat listeners
    document.getElementById('new-chat-btn').addEventListener('click', handleNewChat);
    document.getElementById('user-search').addEventListener('input', handleUserSearch);
    document.getElementById('send-btn').addEventListener('click', handleSendMessage);
    document.getElementById('message-input').addEventListener('keypress', handleMessageInputKeypress);

    // Media listeners
    document.getElementById('file-btn').addEventListener('click', () => document.getElementById('file-input').click());
    document.getElementById('file-input').addEventListener('change', handleFileUpload);
    document.getElementById('voice-btn').addEventListener('click', handleVoiceRecording);
    document.getElementById('video-btn').addEventListener('click', handleVideoRecording);

    // Call listeners
    document.getElementById('voice-call-btn').addEventListener('click', () => alert('Voice call feature coming soon!'));
    document.getElementById('video-call-btn').addEventListener('click', () => alert('Video call feature coming soon!'));

    // Window closing
    window.addEventListener('beforeunload', handleWindowClose);
}

// Authentication functions
async function handleLogin(e) {
    e.preventDefault();
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;

    try {
        const response = await fetch('/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password }),
        });

        const data = await response.json();
        if (data.status === 'success') {
            currentUserId = data.user.id;
            currentUsername = data.user.username;
            showChatPage();
            connectWebSocket();
            loadChats();
            await loadLastChat();
        } else {
            alert(data.message);
        }
    } catch (error) {
        console.error('Login error:', error);
        alert('Login failed');
    }
}

// Add new functions for last chat
function saveLastChat(userId, username) {
    localStorage.setItem('lastChat', JSON.stringify({ id: userId, username }));
}

async function loadLastChat() {
    const lastChat = JSON.parse(localStorage.getItem('lastChat'));
    if (lastChat) {
        await selectChat(lastChat.id, lastChat.username);
    }
}

async function handleRegister(e) {
    e.preventDefault();
    const username = document.getElementById('reg-username').value;
    const password = document.getElementById('reg-password').value;

    try {
        const response = await fetch('/register', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password }),
        });

        const data = await response.json();
        if (data.status === 'success') {
            alert('Registration successful! Please log in.');
            showLoginPage();
        } else {
            alert(data.message);
        }
    } catch (error) {
        alert('Registration failed');
    }
}

// Chat functions
function connectWebSocket() {
    ws = new WebSocket(`ws://${window.location.hostname}:8000/ws/${currentUserId}`);

    ws.onmessage = (event) => {
        const message = JSON.parse(event.data);

        // Display message if in current chat
        if (message.from_user_id === currentChatUserId || message.to_user_id === currentChatUserId) {
            displayMessage(message);
            const messagesContainer = document.getElementById('messages');
            messagesContainer.scrollTop = messagesContainer.scrollHeight;
        }

        // Update chat list to show most recent chat at top
        updateChatList(message);
    };

    function updateChatList(message) {
        const otherUserId = message.from_user_id === currentUserId ? message.to_user_id : message.from_user_id;
        const chatList = document.getElementById('chat-list');
        const existingChat = Array.from(chatList.children).find(chat => chat.dataset.userId === otherUserId.toString());

        if (existingChat) {
            chatList.removeChild(existingChat);
            chatList.insertBefore(existingChat, chatList.firstChild);
        }
    }

    ws.onopen = () => console.log('WebSocket connected');
    ws.onerror = (error) => console.error('WebSocket error:', error);
    ws.onclose = () => setTimeout(connectWebSocket, 1000);
}

async function loadChats() {
    try {
        const response = await fetch('/chats');
        const data = await response.json();

        if (data.status === 'success') {
            const chatList = document.getElementById('chat-list');
            chatList.innerHTML = '';

            data.chats.forEach(chat => {
                const chatElement = createChatElement(chat.id, chat.username);
                chatList.appendChild(chatElement);
            });
        }
    } catch (error) {
        console.error('Failed to load chats:', error);
    }
}

async function handleNewChat() {
    await loadUsers();
    chatModal.show();
}

async function loadUsers() {
    try {
        const response = await fetch('/users');
        const data = await response.json();

        if (data.status === 'success') {
            const usersList = document.getElementById('users-list');
            usersList.innerHTML = '';

            data.users.forEach(user => {
                const userElement = document.createElement('a');
                userElement.href = '#';
                userElement.className = 'list-group-item list-group-item-action';
                userElement.innerHTML = `
                    <div class="d-flex align-items-center">
                        <div class="chat-avatar">${user.username.charAt(0).toUpperCase()}</div>
                        <div>${user.username}</div>
                    </div>
                `;
                userElement.onclick = (e) => {
                    e.preventDefault();
                    startNewChat(user.id, user.username);
                };
                usersList.appendChild(userElement);
            });
        }
    } catch (error) {
        console.error('Failed to load users:', error);
    }
}

function handleUserSearch(e) {
    const searchTerm = e.target.value.toLowerCase();
    const usersList = document.getElementById('users-list').children;

    Array.from(usersList).forEach(user => {
        const username = user.textContent.toLowerCase();
        user.style.display = username.includes(searchTerm) ? '' : 'none';
    });
}

function createChatElement(userId, username) {
    const chatElement = document.createElement('div');
    chatElement.className = 'p-3 border-bottom chat-item';
    chatElement.dataset.userId = userId;
    chatElement.innerHTML = `
        <div class="d-flex align-items-center">
            <div class="chat-avatar">${username.charAt(0).toUpperCase()}</div>
            <div class="chat-name">${username}</div>
        </div>
    `;
    chatElement.onclick = () => selectChat(userId, username);
    return chatElement;
}

async function startNewChat(userId, username) {
    chatModal.hide();
    await selectChat(userId, username);

    const chatList = document.getElementById('chat-list');
    const existingChat = Array.from(chatList.children)
        .find(chat => chat.dataset.userId === userId.toString());

    if (!existingChat) {
        const chatElement = createChatElement(userId, username);
        chatList.appendChild(chatElement);
    }
}

async function selectChat(userId, username) {
    currentChatUserId = userId;
    document.getElementById('current-chat-name').textContent = username;
    saveLastChat(userId, username);

    try {
        const response = await fetch(`/messages/${userId}`);
        const data = await response.json();

        if (data.status === 'success') {
            const messagesContainer = document.getElementById('messages');
            messagesContainer.innerHTML = '';
            data.messages.forEach(message => displayMessage(message));
        }
    } catch (error) {
        console.error('Failed to load messages:', error);
    }
}

// Message handling functions
function handleSendMessage() {
    const messageInput = document.getElementById('message-input');
    const content = messageInput.value.trim();

    if (content && currentChatUserId) {
        const message = {
            from_user_id: currentUserId,
            to_user_id: currentChatUserId,
            content: content,
            message_type: 'text'
        };

        sendMessage(message);
        displayMessage({...message, created_at: new Date().toISOString()});
        messageInput.value = '';
    }
}

async function sendMessage(message) {
    try {
        const response = await fetch('/message', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(message),
        });

        const data = await response.json();
        if (data.status === 'success') {
            ws.send(JSON.stringify(message));
        }
    } catch (error) {
        console.error('Failed to send message:', error);
    }
}

function displayMessage(message) {
    const messageElement = document.createElement('div');
    messageElement.className = `message ${message.from_user_id === currentUserId ? 'message-sent' : 'message-received'}`;

    const timestamp = new Date(message.created_at).toLocaleTimeString();

    messageElement.innerHTML = `
        <div class="message-content">${message.content}</div>
        <div class="message-time text-muted small">${timestamp}</div>
    `;

    document.getElementById('messages').appendChild(messageElement);
}

    const messagesContainer = document.getElementById('messages');
    const messageElement = document.createElement('div');
    messageElement.className = `message ${message.from_user_id === currentUserId ? 'message-sent' : 'message-received'}`;

    switch (message.message_type) {
        case 'text':
            messageElement.textContent = message.content;
            break;
        case 'file':
            const link = document.createElement('a');
            link.href = message.file_path;
            link.textContent = message.content;
            link.target = '_blank';
            messageElement.appendChild(link);
            break;
        case 'voice':
        case 'video':
            const media = document.createElement(message.message_type === 'voice' ? 'audio' : 'video');
            media.src = message.file_path;
            media.controls = true;
            messageElement.appendChild(media);
            break;
    }

    messagesContainer.appendChild(messageElement);
    messagesContainer.scrollTop = messagesContainer.scrollHeight;


// Media handling functions
async function handleFileUpload(e) {
    const file = e.target.files[0];
    if (file && currentChatUserId) {
        const formData = new FormData();
        formData.append('file', file);

        try {
            const response = await fetch('/upload', {
                method: 'POST',
                body: formData
            });

            const data = await response.json();
            if (data.status === 'success') {
                sendMessage({
                    from_user_id: currentUserId,
                    to_user_id: currentChatUserId,
                    content: file.name,
                    message_type: 'file',
                    file_path: data.file_path
                });
            }
        } catch (error) {
            console.error('Failed to upload file:', error);
        }
    }
}

// Media Recording functions
async function handleVoiceRecording() {
    try {
        if (!mediaRecorder) {
            const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
            startRecording(stream, 'voice');
        } else {
            stopRecording();
        }
    } catch (error) {
        console.error('Failed to access microphone:', error);
    }
}

async function handleVideoRecording() {
    try {
        if (!mediaRecorder) {
            const stream = await navigator.mediaDevices.getUserMedia({
                audio: true,
                video: true
            });
            startRecording(stream, 'video');
        } else {
            stopRecording();
        }
    } catch (error) {
        console.error('Failed to access camera:', error);
    }
}

function startRecording(stream, type) {
    recordedChunks = [];
    mediaRecorder = new MediaRecorder(stream);

    mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) {
            recordedChunks.push(e.data);
        }
    };

    mediaRecorder.onstop = async () => {
        const blob = new Blob(recordedChunks, {
            type: type === 'voice' ? 'audio/webm' : 'video/webm'
        });

        const formData = new FormData();
        formData.append('file', blob, `${type}-${Date.now()}.webm`);

        try {
            const response = await fetch('/upload', {
                method: 'POST',
                body: formData
            });

            const data = await response.json();
            if (data.status === 'success' && currentChatUserId) {
                sendMessage({
                    from_user_id: currentUserId,
                    to_user_id: currentChatUserId,
                    content: `${type} message`,
                    message_type: type,
                    file_path: data.file_path
                });
            }
        } catch (error) {
            console.error(`Failed to upload ${type}:`, error);
        }

        stream.getTracks().forEach(track => track.stop());
        mediaRecorder = null;

        // Update button state
        const button = document.getElementById(`${type}-btn`);
        button.classList.remove('btn-danger');
        button.classList.add('btn-outline-secondary');
    };

    mediaRecorder.start();

    // Update button state
    const button = document.getElementById(`${type}-btn`);
    button.classList.remove('btn-outline-secondary');
    button.classList.add('btn-danger');

    // Stop recording after 1 minute
    setTimeout(() => {
        if (mediaRecorder && mediaRecorder.state === 'recording') {
            stopRecording();
        }
    }, 60000);
}

function stopRecording() {
    if (mediaRecorder) {
        mediaRecorder.stop();
    }
}

// Helper functions
function showLoginPage() {
    document.getElementById('register-page').style.display = 'none';
    document.getElementById('login-page').style.display = 'flex';
}

function showRegisterPage() {
    document.getElementById('login-page').style.display = 'none';
    document.getElementById('register-page').style.display = 'flex';
}

// Update showChatPage function
async function showChatPage() {
    document.getElementById('login-page').style.display = 'none';
    document.getElementById('register-page').style.display = 'none';
    document.getElementById('chat-page').style.display = 'block';
    startChatRefresh();
}

// Event handlers
function handleMessageInputKeypress(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSendMessage();
    }
}

function handleWindowClose() {
    if (ws) ws.close();
    if (mediaRecorder && mediaRecorder.state === 'recording') {
        stopRecording();
    }
}