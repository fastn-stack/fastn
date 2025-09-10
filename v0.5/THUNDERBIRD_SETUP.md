# Thunderbird Setup for fastn Email

## Overview

This guide walks through setting up Thunderbird to connect to a fastn rig for sending and receiving emails. Thunderbird has excellent IMAP and STARTTLS support, making it ideal for fastn email testing.

## Prerequisites

- **Thunderbird installed** (download from https://thunderbird.net)
- **fastn-rig running** with known account credentials
- **SMTP and IMAP ports** accessible (default: 2525/1143)

## Step-by-Step Setup

### 1. Start Account Creation

1. **Open Thunderbird**
2. If first time: Skip the existing email provider options
3. If existing installation: **File** → **New** → **Existing Mail Account**

### 2. Account Information

**Enter your fastn account details:**
```
Your name: Alice (or your preferred display name)
Email address: alice@{your_account_id52}.com  
Password: {your_account_password_from_init}
```

**Important**: 
- Replace `{your_account_id52}` with your actual 52-character account ID
- Replace `{your_account_password_from_init}` with your actual password from `fastn-rig init`

### 3. Manual Configuration

1. **Click "Configure manually"** (don't use auto-detection)
2. **Configure the following settings:**

**Incoming (IMAP):**
```
Protocol: IMAP
Hostname: localhost  
Port: 1143 (or your custom FASTN_IMAP_PORT)
SSL: None (we'll enable STARTTLS later)
Authentication: Normal password
Username: alice@{your_account_id52}.com
```

**Outgoing (SMTP):**
```
Hostname: localhost
Port: 2525 (or your custom FASTN_SMTP_PORT)  
SSL: None (we'll enable STARTTLS later)
Authentication: Normal password
Username: alice@{your_account_id52}.com
```

### 4. Test Connection

1. **Click "Re-test"** → Both incoming and outgoing should show green checkmarks
2. **Click "Done"** → Account should be created successfully

**If connection fails:**
- Check that fastn-rig is running in terminal
- Verify port numbers match your FASTN_SMTP_PORT and FASTN_IMAP_PORT
- Check for error messages in fastn-rig terminal output

### 5. Verify Folder Structure

**You should see these folders in Thunderbird:**
- 📁 **INBOX** (for receiving emails)
- 📁 **Sent** (for emails you send)  
- 📁 **Drafts** (for draft emails)
- 📁 **Trash** (for deleted emails)

If folders don't appear, try:
- Right-click account → **Subscribe** → Select all folders
- **File** → **Get Messages** → Refresh folder list

### 6. Send Test Email

**Send email to yourself:**
1. **Click "Write"**
2. **To**: alice@{your_account_id52}.com (send to yourself first)
3. **Subject**: "fastn Email Test"
4. **Body**: "Testing fastn email system with Thunderbird"  
5. **Click "Send"**

**Verify delivery:**
- Check **Sent** folder → Should contain your sent email
- Check **INBOX** → Should receive the email (self-delivery)
- **Open the received email** → Verify content matches

### 7. Enable STARTTLS (Optional)

**For encrypted connections:**

1. **Account Settings** → **Server Settings (IMAP)**
   - **Security**: Change to **STARTTLS**
   - **Port**: Usually stays 1143
   - **Click OK**

2. **Account Settings** → **Outgoing Server (SMTP)**
   - **Security**: Change to **STARTTLS**  
   - **Port**: Usually stays 2525
   - **Click OK**

3. **Certificate Trust** (when prompted):
   - **Accept certificate warning**
   - **Permanently store this exception**
   
**Test encrypted connection:**
- **Send another test email** → Should work with encryption
- **Check connection security** in account settings

## Troubleshooting

### **"Connection refused" Error**
```bash
# Check if fastn-rig is running:
ps aux | grep fastn-rig

# Check if ports are being used:  
lsof -i :2525
lsof -i :1143

# Restart fastn-rig if needed
```

### **"Authentication failed" Error**  
- Verify username is exactly: `alice@{account_id52}.com`
- Verify password matches exactly (copy/paste recommended)
- Check fastn-rig terminal for authentication logs

### **"Certificate not trusted" Error**
1. **Tools** → **Settings** → **Privacy & Security** → **Certificates**
2. **View Certificates** → **Servers tab** → **Add Exception**  
3. **Enter**: `localhost:1143` for IMAP or `localhost:2525` for SMTP
4. **Get Certificate** → **Confirm Security Exception**

### **Emails not appearing**
- **Check folder subscriptions**: Right-click account → **Subscribe**
- **Force refresh**: **File** → **Get Messages**  
- **Check fastn-rig logs**: Look for P2P delivery messages in terminal

## Success Indicators

**✅ Setup Successful When:**
- Thunderbird shows all 4 folders (INBOX, Sent, Drafts, Trash)
- Can send email to yourself and receive it
- No error messages in Thunderbird or fastn-rig terminal
- Email delivery happens within 10 seconds

**🎯 Ready for Cross-Rig Testing:**
Once Thunderbird setup complete, move to setting up second email client for the other rig to test bidirectional email delivery.

## Advanced Features

### **Message Filters**
- Set up filters to organize incoming emails by sender
- Test with emails from different fastn accounts

### **Offline Sync**  
- Configure folder sync settings
- Test reading emails when fastn-rig is offline

### **Multiple Accounts**
- Add multiple fastn accounts to single Thunderbird  
- Test switching between different fastn rigs

This setup provides a complete email client experience with fastn's P2P email infrastructure!