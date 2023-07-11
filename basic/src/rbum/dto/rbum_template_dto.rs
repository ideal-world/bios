
// public class RbumTemplateDto {

//     @Data
//     @SuperBuilder
//     @NoArgsConstructor
//     @AllArgsConstructor
//     @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
//     public static class RbumTemplateAddReq {

//         @Schema(description = "资源作用域级别")
//         @Enumerated
//         protected RbumScopeLevelKind scopeLevel;

//         @Size(max = 255)
//         @Schema(description = "编码")
//         private String code;

//         @NotNull
//         @Size(max = 255)
//         @Schema(description = "名称")
//         private String name;

//         @Size(max = 2000)
//         @Schema(description = "说明")
//         @Builder.Default
//         private String note = "";

//         @Size(max = 1000)
//         @Schema(description = "图标")
//         @Builder.Default
//         private String icon = "";

//         @Schema(description = "排序")
//         @Builder.Default
//         private Integer sort = 0;

//         @Schema(description = "是否禁用")
//         @Builder.Default
//         private Boolean disabled = true;

//         @Schema(description = "参数")
//         @Builder.Default
//         private String variables = "";

//     }

//     @Data
//     @SuperBuilder
//     @NoArgsConstructor
//     @AllArgsConstructor
//     @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
//     public static class RbumTemplateModifyReq {

//         @Schema(description = "资源作用域级别")
//         @Enumerated
//         protected RbumScopeLevelKind scopeLevel;

//         @Size(max = 255)
//         @Schema(description = "编码")
//         private String code;

//         @Size(max = 255)
//         @Schema(description = "名称")
//         private String name;

//         @Size(max = 2000)
//         @Schema(description = "说明")
//         private String note;

//         @Size(max = 1000)
//         @Schema(description = "图标")
//         private String icon;

//         @Schema(description = "排序")
//         private Integer sort;

//         @Schema(description = "是否禁用")
//         private Boolean disabled;

//         @Schema(description = "参数")
//         private String variables;

//     }

//     @EqualsAndHashCode(callSuper = true)
//     @Data
//     @SuperBuilder
//     @NoArgsConstructor
//     @AllArgsConstructor
//     @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
//     public static class RbumTemplateSummaryResp extends RbumScopeSummaryResp {

//         @Schema(description = "编码")
//         private String code;

//         @Schema(description = "名称")
//         private String name;

//         @Schema(description = "说明")
//         private String note;

//         @Schema(description = "图标")
//         private String icon;

//         @Schema(description = "排序")
//         private Integer sort;

//         @Schema(description = "是否禁用")
//         private Boolean disabled;

//         @Schema(description = "参数")
//         private String variables;


//     }

//     @EqualsAndHashCode(callSuper = true)
//     @Data
//     @SuperBuilder
//     @NoArgsConstructor
//     @AllArgsConstructor
//     @JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
//     public static class RbumTemplateDetailResp extends RbumScopeDetailResp {

//         @Schema(description = "编码")
//         private String code;

//         @Schema(description = "名称")
//         private String name;

//         @Schema(description = "说明")
//         private String note;

//         @Schema(description = "图标")
//         private String icon;

//         @Schema(description = "排序")
//         private Integer sort;

//         @Schema(description = "是否禁用")
//         private Boolean disabled;

//         @Schema(description = "参数")
//         private String variables;


//     }
// }
