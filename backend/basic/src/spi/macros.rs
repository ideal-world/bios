//! SPI macros

#[macro_export]
macro_rules! spi_service_call {
    ($mod:path, $fun:ident, $funs:ident, $ctx:ident, $inst:ident, @args: {$($args: ident),*}) => {
        {
            use $mod::*;
            $fun($($args,)* $funs, $ctx, $inst).await
        }
    };
}

#[macro_export]
macro_rules! spi_dispatch_function {
    (
        $service:ident,
        $funs:ident, $ctx:ident, $inst:ident,
        @dispatch: {
            $(
                $(#[$attr:meta])*
                $code:pat=>$mod:path,
            )*
        },
        @args: $args: tt
    ) => {
        match $inst.kind_code() {
            $(
                $(#[$attr])*
                $code => $crate::spi_service_call!($mod, $service, $funs, $ctx, $inst, @args: $args),
            )*
            kind_code => Err($funs.bs_not_implemented(kind_code)),
        }

    };
}

/// SPI request dispatch macro
/// SPI请求分发宏
///
/// Used to generate request dispatch code in the SPI service
/// 用于批量生成请求分发代码
#[macro_export]
macro_rules! spi_dispatch_service {
    (
        // Whether it is a managed request
        // 是否是管理模式
        @mgr: $mgr: expr,
        // Backend service initialization method, called when the backend service instance is not initialized
        // 后端服务初始化方法，当后端服务实例未初始化时调用
        @init: $init: expr,
        // Service object to dispatch to
        // 分发到的服务对象
        @dispatch: $dispatch:tt,
        // Collection of request methods
        // 请求的方法集合
        @method: {
            $(
                $(#[$attr:meta])*
                $service:ident($($arg: ident: $type: ty),*) -> $ret:ty;
            )*
        }

    ) => {
        $(
            $(#[$attr])*
            pub async fn $service($($arg: $type,)* funs: &tardis::TardisFunsInst, ctx: &tardis::basic::dto::TardisContext) -> $ret {
                let arc_inst = funs.init(ctx, $mgr, $init).await?;
                let inst = arc_inst.as_ref();
                $crate::spi_dispatch_function!($service, funs, ctx, inst, @dispatch: $dispatch, @args: {$($arg),*})
            }
        )*
    };
}
