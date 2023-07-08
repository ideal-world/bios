/*
public class ReachMsgTemplateDto {

    @EqualsAndHashCode(callSuper = true)
    @Data
    @SuperBuilder
    @NoArgsConstructor
    @AllArgsConstructor
    @Schema(title = "添加用户触达消息模板请求")
    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
    public static class ReachMsgTemplateAddReq extends RbumTemplateDto.RbumTemplateAddReq {

        @NotNull
        @Schema(description = "用户触达等级类型", example = "NORMAL")
        @Enumerated(EnumType.STRING)
        private ReachLevelKind levelKind;

        @NotNull
        @Size(max = 255)
        @Schema(description = "主题")
        private String topic;

        @NotNull
        @Schema(description = "内容", example = "内容")
        private String content;

        @NotNull
        @Schema(description = "确认超时时间", example = "10")
        private Integer timeoutSec;

        @NotNull
        @Schema(description = "确认超时策略", example = "NORMAL")
        @Enumerated(EnumType.STRING)
        private ReachTimeoutStrategyKind timeoutStrategy;

        @NotNull
        @Schema(description = "关联的触达通道", example = "SMS")
        private ReachChannelKind relReachChannel;

        @NotNull
        @Schema(description = "模板类型", example = "是否是验证码类型")
        @Enumerated(EnumType.STRING)
        private ReachTemplateKind kind;

        @Size(max = 255)
        @Schema(description = "用户触达验证码策略Id", example = "用户触达验证码策略Id")
        private String relReachVerifyCodeStrategyId;

        @Size(max = 255)
        @Schema(description = "第三方插件-模板Id", example = "第三方插件-模板Id")
        @Builder.Default
        private String smsTemplateId = "";

        @Size(max = 255)
        @Schema(description = "第三方插件-签名", example = "第三方插件-签名")
        @Builder.Default
        private String smsSignature = "";

        @Size(max = 255)
        @Schema(description = "第三方插件-短信发送方的号码", example = "第三方插件-短信发送方的号码")
        @Builder.Default
        private String smsFrom = "";
    }

    @EqualsAndHashCode(callSuper = true)
    @Data
    @SuperBuilder
    @NoArgsConstructor
    @AllArgsConstructor
    @Schema(title = "修改用户触达消息模板请求")
    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
    public static class ReachMsgTemplateModifyReq extends RbumTemplateDto.RbumTemplateModifyReq {

        @Schema(description = "用户触达等级类型", example = "NORMAL")
        @Enumerated(EnumType.STRING)
        private ReachLevelKind levelKind;

        @Size(max = 255)
        @Schema(description = "主题")
        private String topic;

        @Schema(description = "内容", example = "内容")
        private String content;

        @Schema(description = "确认超时时间", example = "10")
        private Integer timeoutSec;

        @Schema(description = "确认超时策略", example = "NORMAL")
        @Enumerated(EnumType.STRING)
        private ReachTimeoutStrategyKind timeoutStrategy;

        @Schema(description = "关联的触达通道", example = "SMS")
        private ReachChannelKind relReachChannel;

        @Schema(description = "模板类型", example = "是否是验证码类型")
        @Enumerated(EnumType.STRING)
        private ReachTemplateKind kind;

        @Size(max = 255)
        @Schema(description = "用户触达验证码策略Id", example = "用户触达验证码策略Id")
        private String relReachVerifyCodeStrategyId;

        @Size(max = 255)
        @Schema(description = "第三方插件-模板Id", example = "第三方插件-模板Id")
        private String smsTemplateId;

        @Size(max = 255)
        @Schema(description = "第三方插件-签名", example = "第三方插件-签名")
        private String smsSignature;

        @Size(max = 255)
        @Schema(description = "第三方插件-短信发送方的号码", example = "第三方插件-短信发送方的号码")
        private String smsFrom;

    }

    @EqualsAndHashCode(callSuper = true)
    @Data
    @SuperBuilder
    @NoArgsConstructor
    @AllArgsConstructor
    @Schema(title = "用户触达消息模板过滤请求")
    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
    public static class ReachMsgTemplateFilterReq extends RbumFilterDto.RbumItemBasicFilterReq {

        @Size(max = 512)
        @Schema(description = "关联的触达通道", example = "SMS")
        private ReachChannelKind relReachChannel;

        @Schema(description = "用户触达等级类型", example = "NORMAL")
        @Enumerated(EnumType.STRING)
        private ReachLevelKind levelKind;

        @Schema(description = "模板类型", example = "是否是验证码类型")
        @Enumerated(EnumType.STRING)
        private ReachTemplateKind kind;


    }

    @EqualsAndHashCode(callSuper = true)
    @Data
    @SuperBuilder
    @NoArgsConstructor
    @AllArgsConstructor
    @Schema(title = "用户触达消息模板概要信息")
    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
    public static class ReachMsgTemplateSummaryResp extends RbumTemplateDto.RbumTemplateSummaryResp {

        @NotNull
        @Schema(description = "用户触达等级类型", example = "NORMAL")
        @Enumerated(EnumType.STRING)
        private ReachLevelKind levelKind;

        @Size(max = 255)
        @Schema(description = "主题")
        private String topic;

        @NotNull
        @Schema(description = "内容", example = "内容")
        private String content;

        @NotNull
        @Schema(description = "确认超时时间", example = "10")
        private Integer timeoutSec;

        @NotNull
        @Schema(description = "确认超时策略", example = "NORMAL")
        @Enumerated(EnumType.STRING)
        private ReachTimeoutStrategyKind timeoutStrategy;

        @NotNull
        @Size(max = 512)
        @Schema(description = "关联的触达通道", example = "SMS")
        private ReachChannelKind relReachChannel;

        @Schema(description = "模板类型", example = "是否是验证码类型")
        @Enumerated(EnumType.STRING)
        private ReachTemplateKind kind;

        @NotNull
        @Size(max = 255)
        @Schema(description = "用户触达验证码策略Id", example = "用户触达验证码策略Id")
        private String relReachVerifyCodeStrategyId;

        @Size(max = 255)
        @Schema(description = "第三方插件-模板Id", example = "第三方插件-模板Id")
        private String smsTemplateId;

        @Size(max = 255)
        @Schema(description = "第三方插件-签名", example = "第三方插件-签名")
        private String smsSignature;

        @Size(max = 255)
        @Schema(description = "第三方插件-短信发送方的号码", example = "第三方插件-短信发送方的号码")
        private String smsFrom;

    }

    @EqualsAndHashCode(callSuper = true)
    @Data
    @SuperBuilder
    @NoArgsConstructor
    @AllArgsConstructor
    @Schema(title = "用户触达消息模板详细信息")
    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
    public static class ReachMsgTemplateDetailResp extends RbumTemplateDto.RbumTemplateDetailResp {

        @NotNull
        @Schema(description = "用户触达等级类型", example = "NORMAL")
        @Enumerated(EnumType.STRING)
        private ReachLevelKind levelKind;

        @Size(max = 255)
        @Schema(description = "主题")
        private String topic;

        @NotNull
        @Schema(description = "内容", example = "内容")
        private String content;

        @NotNull
        @Schema(description = "确认超时时间", example = "10")
        private Integer timeoutSec;

        @NotNull
        @Schema(description = "确认超时策略", example = "NORMAL")
        @Enumerated(EnumType.STRING)
        private ReachTimeoutStrategyKind timeoutStrategy;

        @NotNull
        @Size(max = 512)
        @Schema(description = "关联的触达通道", example = "SMS")
        private ReachChannelKind relReachChannel;

        @Schema(description = "模板类型", example = "是否是验证码类型")
        @Enumerated(EnumType.STRING)
        private ReachTemplateKind kind;

        @NotNull
        @Size(max = 255)
        @Schema(description = "用户触达验证码策略Id", example = "用户触达验证码策略Id")
        private String relReachVerifyCodeStrategyId;

        @Size(max = 255)
        @Schema(description = "第三方插件-模板Id", example = "第三方插件-模板Id")
        private String smsTemplateId;

        @Size(max = 255)
        @Schema(description = "第三方插件-签名", example = "第三方插件-签名")
        private String smsSignature;

        @Size(max = 255)
        @Schema(description = "第三方插件-短信发送方的号码", example = "第三方插件-短信发送方的号码")
        private String smsFrom;

    }

}



 */

use bios_basic::rbum::dto::rbum_safe_dto::{RbumSafeSummaryResp, RbumSafeDetailResp};
use tardis::web::poem_openapi;

use crate::dto::*;

#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgTemplateAddReq {
    // #[oai(flatten)]
    // rbum_template_add_req: RbumTemplateAddReq,
    /// 用户触达等级类型
    level_kind: ReachLevelKind,
    /// 主题
    #[oai(validator(max_length = "255"))]
    topic: String,
    /// 内容
    #[oai(validator(max_length = "2000"))]
    content: String,
    /// 确认超时时间
    timeout_sec: i32,
    /// 确认超时策略
    timeout_strategy: ReachTimeoutStrategyKind,
    /// 关联的触达通道
    rel_reach_channel: ReachChannelKind,
    /// 模板类型
    kind: ReachTemplateKind,
    /// 用户触达验证码策略Id
    #[oai(validator(max_length = "255"))]
    rel_reach_verify_code_strategy_id: String,
    /// 第三方插件-模板Id
    #[oai(validator(max_length = "255"))]
    #[oai(default)]
    sms_template_id: String,
    /// 第三方插件-签名
    #[oai(validator(max_length = "255"))]
    #[oai(default)]
    sms_signature: String,
    /// 第三方插件-短信发送方的号码
    #[oai(validator(max_length = "255"))]
    #[oai(default)]
    sms_from: String,
}