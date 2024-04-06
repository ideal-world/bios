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

#[macro_export]
macro_rules! spi_dispatch_service {
    (
        // mgr
        @mgr: $mgr: expr,
        // init fun
        @init: $init: expr,
        // dispatcher
        @dispatch: $dispatch:tt,
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
