import smtplib

server = smtplib.SMTP('127.0.0.1', 9090)
server.sendmail('siddhant@localhost', 'amitu@localhost', 'This is a test email')
server.quit()
