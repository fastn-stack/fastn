# fastn Apple Mail Testing Environment

**Generated from commit**: `95d4ebbfe38dda941c0670a7d54d9e64619e4071`  
**Security**: Using secure `.fastn` email addresses

## Quick Start

1. **Create environment**: `./setup-fastn-email.sh` (from parent directory)
2. **Start servers**: `./start-servers.sh`  
3. **Setup Apple Mail**: `osascript setup-apple-mail.scpt`
4. **Test automation**: `osascript automated-mail-test.scpt`

## Available Scripts

### **Setup Scripts**
- `setup-apple-mail.scpt` - Set up Alice's account in Apple Mail
- `setup-apple-mail-bob.scpt` - Set up Bob's account in Apple Mail
- `start-servers.sh` - Start both fastn-rig servers

### **Testing Scripts**  
- `check-mail-status.scpt` - Check status of all fastn accounts
- `send-test-email.scpt` - Send test email between accounts
- `refresh-and-check.scpt` - Force mail refresh and check results
- `automated-mail-test.scpt` - Full automated bidirectional test

### **Advanced Automation**
- `test-apple-mail-automation.scpt` - Comprehensive test with timing analysis

## Account Configuration

### Alice
- **Email**: `alice@71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0.fastn`
- **IMAP**: localhost:8143 (No encryption)
- **SMTP**: localhost:8587 (No encryption)
- **Password**: `DI*l4qHBjdGl$J7t`

### Bob
- **Email**: `bob@2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0.fastn`
- **IMAP**: localhost:8144 (No encryption)  
- **SMTP**: localhost:8588 (No encryption)
- **Password**: `Q0MilZ7sjJB74mDq`

## Apple Script Capabilities

✅ **Account Setup**: Automated Apple Mail account creation  
✅ **Mail Status**: Check message counts across folders
✅ **Send Emails**: Programmatically compose and send test emails
✅ **Delivery Verification**: Check for successful P2P delivery
✅ **Bidirectional Testing**: Test both Alice→Bob and Bob→Alice
✅ **Timing Analysis**: Monitor P2P delivery performance
✅ **Error Reporting**: Detailed error messages and troubleshooting

## Testing Workflow

1. **Environment Setup**: Run setup scripts to create accounts
2. **Manual Verification**: Send emails via Apple Mail UI to verify basic functionality  
3. **Automated Testing**: Use automation scripts for systematic validation
4. **Performance Analysis**: Monitor delivery timing and success rates

## Security Features

- ✅ **Secure .fastn addresses**: Prevents domain hijacking attacks
- ✅ **P2P delivery**: No reliance on external email providers
- ✅ **Local IMAP/SMTP**: Email client connects to local fastn-rig
- ✅ **Cryptographic identity**: Account IDs based on public keys

## Troubleshooting

- **Server logs**: `tail -f ~/fastn-email/*_server.log`
- **Account status**: `osascript check-mail-status.scpt`
- **Manual test**: Send email via Apple Mail UI first
- **Server restart**: Kill servers and run `./start-servers.sh` again

---
*Ready for comprehensive Apple Mail testing with fastn P2P email! 🚀*