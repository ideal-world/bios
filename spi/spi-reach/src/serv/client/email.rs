/* /*
 * Copyright 2022. the original author or authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

 package tech.starsys.reach.notification;

 import cn.hutool.core.date.DateUtil;
 import cn.hutool.core.util.StrUtil;
 import com.ecfront.dew.common.Resp;
 import com.ecfront.dew.common.exception.RTIOException;
 import com.sun.mail.util.MailSSLSocketFactory;
 import group.idealworld.dew.core.basic.resp.StandardResp;
 import lombok.extern.slf4j.Slf4j;
 import tech.starsys.common.rbum.RbumConstants;
 import tech.starsys.common.rbum.helper.RbumScopeHelper;
 import tech.starsys.reach.config.MailConfig;
 import tech.starsys.reach.config.ReachNotificationConfig;
 import tech.starsys.reach.dto.ReachMsgLogDto;
 import tech.starsys.reach.enumeration.ReachDndStrategyKind;
 import tech.starsys.reach.integration.iam.IamIntegrationService;
 
 import javax.mail.*;
 import javax.mail.internet.InternetAddress;
 import javax.mail.internet.MimeMessage;
 import java.util.Map;
 import java.util.Optional;
 import java.util.Properties;
 import java.util.Set;
 import java.util.regex.Pattern;
 
 import static tech.starsys.reach.config.ReachNotificationConfig.REACH_MSG_LOG_SERVICE;
 
 @Slf4j
 public class MailChannel {
 
     private static final Pattern EXTRACT_R = Pattern.compile("(\\{.+?})");
 
     private static final String MAIL_V_CODE = "MailVCode";
 
     private static Session initSession(MailConfig mailConfig) {
         try {
             String host = mailConfig.getHost();
             String port = mailConfig.getPort();
             String secure = Optional.ofNullable(mailConfig.getSecure()).orElse("");
             Properties props = new Properties();
             // 开启debug调试
             props.setProperty("mail.debug", "true");
             // 设置邮件服务器主机名
             props.setProperty("mail.smtp.host", host);
             props.setProperty("mail.smtp.port", port);
             // 发送服务器需要身份验证
             props.put("mail.smtp.auth", "true");
             MailSSLSocketFactory sf = new MailSSLSocketFactory();
             sf.setTrustAllHosts(true);
             props.put("mail.smtp.ssl.enable", "true");
             props.put("mail.smtp.ssl.socketFactory", sf);
             return Session.getDefaultInstance(props, new Authenticator() {
                 @Override
                 public PasswordAuthentication getPasswordAuthentication() {
                     return new PasswordAuthentication(mailConfig.getUsername(), mailConfig.getPassword());
                 }
             });
         } catch (NullPointerException ex) {
             log.error("Notify Mail channel init error,missing [from] [host] [port] [username] [password] parameters", ex);
             throw ex;
         } catch (Exception e) {
             log.error(e.getMessage());
             return null;
         }
     }
 
     private Message beforeMessage(String title,String content) throws Exception{
         var mailConfig = ReachNotificationConfig.MAIL_CONFIG;
         var session = initSession(mailConfig);
         assert session != null;
         var transport = session.getTransport();
         transport.connect(mailConfig.getHost(), mailConfig.getUsername(), mailConfig.getPassword());
         Message msg = new MimeMessage(session);
         msg.setSubject(title);
         msg.setText(content);
         msg.setFrom(new InternetAddress(mailConfig.getFrom()));
         return msg;
     }
 
     public static Resp<String> send(String content, String title, Set<String> receivers) {
         try {
             var mailConfig = ReachNotificationConfig.MAIL_CONFIG;
             var session = initSession(mailConfig);
             assert session != null;
             Message msg = new MimeMessage(session);
             msg.setSubject(title);
             msg.setText(content);
             msg.setFrom(new InternetAddress(mailConfig.getFrom()));
             for (var receiver : receivers) {
                 msg.setRecipients(Message.RecipientType.TO,
                         InternetAddress.parse(receiver));
                 Transport.send(msg);
             }
             return Resp.success("");
         } catch (Exception e) {
             e.printStackTrace();
             return StandardResp.badRequest("mail-send", e.getMessage());
         }
     }
 
     public static Resp<String> accountSend(String relReachMessageId, String ownPaths, String content, String title, Set<String> accountIds, Map<String, String> replace) {
         try {
             var session = initSession(ReachNotificationConfig.MAIL_CONFIG);
             assert session != null;
             for (var accountId : accountIds) {
                 accountSend(relReachMessageId, ownPaths, content, title, accountId, session, replace);
             }
             return Resp.success("");
         } catch (Exception e) {
             return StandardResp.badRequest("mail-send", e.getMessage());
         }
     }
 
     public static void accountSend(
             String relReachMessageId,
             String ownPaths,
             String content,
             String title,
             String accountId,
             Session session,
             Map<String, String> replace
     ) {
         var startTime = DateUtil.date();
         var failure = Boolean.FALSE;
         var failMessage = "";
         try {
             Message msg = new MimeMessage(session);
             msg.setSubject(title);
             msg.setText(contentReplace(content, replace));
             msg.setFrom(new InternetAddress(ReachNotificationConfig.MAIL_CONFIG.getFrom()));
             var accountAggResp =
                     IamIntegrationService.getAccount(accountId,
                             RbumScopeHelper.getPrePaths(RbumConstants.RBUM_SCOPE_LEVEL_TENANT.getValue(),
                                     ownPaths).orElse(""));
             if (StrUtil.isNotBlank(accountAggResp.getCerts().get(MAIL_V_CODE))) {
                 msg.setRecipients(Message.RecipientType.TO,
                         InternetAddress.parse(accountAggResp.getCerts().get(MAIL_V_CODE)));
                 Transport.send(msg);
             } else {
                 log.error("Notify Mail channel send error,missing [MailVCode] parameters");
                 throw new RTIOException("Notify Mail channel send error,missing [MailVCode] parameters");
             }
         } catch (RTIOException re) {
             failure = Boolean.TRUE;
             failMessage = re.getMessage();
         } catch (Exception e) {
             failure = Boolean.TRUE;
             failMessage = e.getMessage();
             throw new RTIOException(e.getMessage());
         } finally {
             REACH_MSG_LOG_SERVICE.addRbum(ReachMsgLogDto.ReachMsgLogAddReq
                     .builder()
                     .relAccountId(accountId)
                     .relReachMessageId(relReachMessageId)
                     .dndTime("")
                     .dndStrategy(ReachDndStrategyKind.IGNORE)
                     .startTime(startTime)
                     .endTime(DateUtil.date())
                     .failure(failure)
                     .failMessage(failMessage)
                     .build());
         }
     }
 
     public static String contentReplace(String content, Map<String, String> values) {
         var new_content = content;
         var matcher = EXTRACT_R.matcher(content);
         while (matcher.find()) {
             var mat = matcher.group();
             var key = mat.substring(1, mat.length() - 1);
             if (values.containsKey(key)) {
                 new_content = new_content.replace(mat, values.get(key));
             }
         }
         return new_content;
     }
 } */
 

pub struct MailClient {
    
}