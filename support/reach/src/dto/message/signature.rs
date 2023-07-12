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

use bios_basic::rbum::dto::{rbum_safe_dto::{RbumSafeSummaryResp, RbumSafeDetailResp}, rbum_filer_dto::RbumBasicFilterReq};
use serde::Serialize;
use tardis::{web::poem_openapi, db::sea_orm::{FromQueryResult, self}, chrono::{DateTime, Utc}};

use crate::dto::*;
/// 添加用户触达签名请求
#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgSignatureAddReq {
    /// 名称
    #[oai(validator(max_length = "255"))]
    pub name: String,
    /// 说明
    #[oai(validator(max_length = "2000"))]
    pub note: String,
    /// 内容
    #[oai(validator(max_length = "2000"))]
    pub content: String,
    /// 来源
    #[oai(validator(max_length = "255"))]
    pub source: String,
    pub rel_reach_channel: ReachChannelKind,
}
/// 修改用户触达签名请求
#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgSignatureModifyReq {
    /// 名称
    #[oai(validator(max_length = "255"))]
    pub name: Option<String>,
    /// 说明
    #[oai(validator(max_length = "2000"))]
    pub note: Option<String>,
    /// 内容
    #[oai(validator(max_length = "2000"))]
    pub content: Option<String>,
    /// 来源
    #[oai(validator(max_length = "255"))]
    pub source: Option<String>,
    pub rel_reach_channel: Option<ReachChannelKind>,
}

/// 用户触达签名过滤请求
#[derive(Debug, poem_openapi::Object)]
pub struct ReachMsgSignatureFilterReq {
    #[oai(flatten)]
    pub base_filter: RbumBasicFilterReq,
    /// 名称
    #[oai(validator(min_length = "1", max_length = "255"))]
    pub name: String,
    /// 关联的触达通道
    pub rel_reach_channel: Option<ReachChannelKind>,
}

#[derive(Debug, poem_openapi::Object, sea_orm::FromQueryResult, Serialize)]
pub struct ReachMsgSignatureSummaryResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub name: String,
    pub note: String,
    pub content: String,
    pub source: String,
    pub rel_reach_channel: ReachChannelKind,
}

#[derive(Debug, poem_openapi::Object, sea_orm::FromQueryResult, Serialize)]
pub struct ReachMsgSignatureDetailResp {
    pub id: String,
    pub own_paths: String,
    pub owner: String,
    pub owner_name: String,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub name: String,
    pub note: String,
    pub content: String,
    pub source: String,
    pub rel_reach_channel: ReachChannelKind,
}