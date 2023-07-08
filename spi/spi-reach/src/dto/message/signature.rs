/*
public class ReachMsgSignatureDto {

    @Data
    @SuperBuilder
    @NoArgsConstructor
    @AllArgsConstructor
    @Schema(title = "添加用户触达签名请求")
    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
    public static class ReachMsgSignatureAddReq {

        @NotNull
        @Size(max = 255)
        @Schema(description = "名称", example = "名称")
        private String name;

        @NotNull
        @Size(max = 2000)
        @Schema(description = "说明", example = "说明")
        private String note;

        @NotNull
        @Size(max = 2000)
        @Schema(description = "内容", example = "hello")
        private String content;

        @NotNull
        @Size(max = 255)
        @Schema(description = "来源", example = "来源")
        private String source;

        @NotNull
        @Schema(description = "关联的触达通道", example = "SMS")
        @Enumerated(EnumType.STRING)
        private ReachChannelKind relReachChannel;

    }

    @Data
    @SuperBuilder
    @NoArgsConstructor
    @AllArgsConstructor
    @Schema(title = "修改用户触达签名请求")
    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
    public static class ReachMsgSignatureModifyReq {

        @Size(max = 255)
        @Schema(description = "名称", example = "名称")
        private String name;

        @Size(max = 2000)
        @Schema(description = "说明", example = "说明")
        private String note;


        @Size(max = 2000)
        @Schema(description = "内容", example = "hello")
        private String content;

        @Size(max = 255)
        @Schema(description = "来源", example = "来源")
        private String source;


        @Schema(description = "关联的触达通道", example = "SMS")
        @Enumerated(EnumType.STRING)
        private ReachChannelKind relReachChannel;

    }

    @EqualsAndHashCode(callSuper = true)
    @Data
    @SuperBuilder
    @NoArgsConstructor
    @AllArgsConstructor
    @Schema(title = "用户触达签名过滤请求")
    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
    public static class ReachMsgSignatureFilterReq extends RbumFilterDto.RbumItemBasicFilterReq {

        @NotNull
        @Size(max = 255)
        @Schema(description = "名称", example = "名称")
        private String name;

        @Size(max = 512)
        @Schema(description = "关联的触达通道", example = "SMS")
        private ReachChannelKind relReachChannel;

    }

    @EqualsAndHashCode(callSuper = true)
    @Data
    @SuperBuilder
    @NoArgsConstructor
    @AllArgsConstructor
    @Schema(title = "用户触达签名概要信息")
    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
    public static class ReachMsgSignatureSummaryResp extends RbumSafeSummaryResp {

        @Size(max = 255)
        @Schema(description = "名称", example = "名称")
        private String name;

        @Size(max = 2000)
        @Schema(description = "说明", example = "说明")
        private String note;

        @Size(max = 2000)
        @Schema(description = "内容", example = "hello")
        private String content;

        @Size(max = 255)
        @Schema(description = "来源", example = "来源")
        private String source;

        @Size(max = 512)
        @Schema(description = "关联的触达通道", example = "SMS")
        private ReachChannelKind relReachChannel;

    }

    @EqualsAndHashCode(callSuper = true)
    @Data
    @SuperBuilder
    @NoArgsConstructor
    @AllArgsConstructor
    @Schema(title = "用户触达签名详细信息")
    @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
    public static class ReachMsgSignatureDetailResp extends RbumSafeDetailResp {

        @Size(max = 255)
        @Schema(description = "名称", example = "名称")
        private String name;

        @Size(max = 2000)
        @Schema(description = "说明", example = "说明")
        private String note;

        @Size(max = 2000)
        @Schema(description = "内容", example = "hello")
        private String content;

        @Size(max = 255)
        @Schema(description = "来源", example = "来源")
        private String source;

        @Size(max = 512)
        @Schema(description = "关联的触达通道", example = "SMS")
        private ReachChannelKind relReachChannel;

    }

}
 */

// convert from java

use bios_basic::rbum::dto::rbum_safe_dto::{RbumSafeSummaryResp, RbumSafeDetailResp};
use tardis::web::poem_openapi;

use crate::dto::*;
/// 添加用户触达签名请求
#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgSignatureAddReq {
    /// 名称
    #[oai(validator(max_length = "255"))]
    name: String,
    /// 说明
    #[oai(validator(max_length = "2000"))]
    note: String,
    /// 内容
    #[oai(validator(max_length = "2000"))]
    content: String,
    /// 来源
    #[oai(validator(max_length = "255"))]
    source: String,
    rel_reach_channel: ReachChannelKind,
}
/// 修改用户触达签名请求
#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgSignatureModifyReq {
    /// 名称
    #[oai(validator(max_length = "255"))]
    name: String,
    /// 说明
    #[oai(validator(max_length = "2000"))]
    note: String,
    /// 内容
    #[oai(validator(max_length = "2000"))]
    content: String,
    /// 来源
    #[oai(validator(max_length = "255"))]
    source: String,
    rel_reach_channel: ReachChannelKind,
}

/// 用户触达签名过滤请求
#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgSignatureFilterReq {
    /// 名称
    name: String,
    /// 关联的触达通道
    rel_reach_channel: ReachChannelKind,
}

#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgSignatureSummaryResp {
    #[oai(flatten)]
    rbum_safe_summary_resp: RbumSafeSummaryResp,
    name: String,
    note: String,
    content: String,
    source: String,
    rel_reach_channel: ReachChannelKind,
}

#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgSignatureDetailResp {
    #[oai(flatten)]
    rbum_safe_detail_resp: RbumSafeDetailResp,
    name: String,
    note: String,
    content: String,
    source: String,
    rel_reach_channel: ReachChannelKind,
}