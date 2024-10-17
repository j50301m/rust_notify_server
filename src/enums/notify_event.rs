use kgs_err::models::status::Status as KgsStatus;
use sea_orm::{entity::prelude::*, strum::Display};

use super::Platform;

/// 通知類型
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Display)]
#[sea_orm(rs_type = "i64", db_type = "BigInteger")]
pub enum NotifyEvent {
    #[strum(to_string = "NormalInfo")]
    NormalInfo = 1, // 一般通知
    #[strum(to_string = "LoginAnomaly")]
    LoginAnomaly = 2, // 登入異常
    #[strum(to_string = "RegisterSuccess")]
    RegisterSuccess = 3, // 註冊成功
    #[strum(to_string = "DepositSuccess")]
    DepositSuccess = 4, // 入金成功
    #[strum(to_string = "WithdrawSuccess")]
    WithdrawSuccess = 5, // 出金成功
    #[strum(to_string = "WithdrawFail")]
    WithdrawFail = 6, // 出金失敗
    #[strum(to_string = "KycVerifySuccess")]
    KycVerifySuccess = 7, // KYC 通過
    #[strum(to_string = "KycVerifyFail")]
    KycVerifyFail = 8, // KYC 失敗
    #[strum(to_string = "KycReverify")]
    KycReverify = 9, // KYC 重新驗證
    #[strum(to_string = "CreditCardVerifySuccess")]
    CreditCardVerifySuccess = 10, // 信用卡驗證成功
    #[strum(to_string = "CreditCardVerifyFail")]
    CreditCardVerifyFail = 11, // 信用卡驗證失敗
    #[strum(to_string = "LoginWarning")]
    LoginWarning = 12, // 登入警告 (密碼錯誤次數過多)
    #[strum(to_string = "UpdateProfileSuccess")]
    UpdateProfileSuccess = 13, // 更新個人資料成功
    #[strum(to_string = "LoginSuccess")]
    LoginSuccess = 14, // 登入成功
    #[strum(to_string = "VerifyEmail")]
    VerifyEmail = 15, // 驗證信箱
    #[strum(to_string = "VerifyPhone")]
    VerifyPhone = 16, // 驗證手機
    #[strum(to_string = "BackstageVerifyKyc")]
    BackstageVerifyKyc = 17, // 後台審核 KYC
    #[strum(to_string = "BackStageVerifyWithdraw")]
    BackstageVerifyWithdraw = 18, // 後台審核出金
    #[strum(to_string = "BackStageVerifyDeposit")]
    BackstageVerifyDeposit = 19, // 後台審核入金
    #[strum(to_string = "BackstageVerifyCreditCard")]
    BackstageVerifyCreditCard = 20, // 後台審核信用卡
    #[strum(to_string = "NewEventOnline")]
    NewEventOnline = 21, // 新活動上線
    #[strum(to_string = "VipLevelUp")]
    VipLevelUp = 22, // VIP等級提升
    #[strum(to_string = "BonusExpiration")]
    BonusExpiration = 23, // 獎金到期
    #[strum(to_string = "EventCompletion")]
    EventCompletion = 24, // 活動完成
    #[strum(to_string = "ReceiveTips")]
    ReceiveTips = 25, // 收到小費
    #[strum(to_string = "GiveTips")]
    GiveTips = 26, // 給予小費
    #[strum(to_string = "ReceiveBirthdayBonus")]
    ReceiveBirthdayBonus = 27, // 收到生日獎金
    #[strum(to_string = "ActivitySerialNumber")]
    ActivitySerialNumber = 28, // 收到活動序號
    #[strum(to_string = "ReceiveBonus")]
    ReceiveBonus = 29, // 收到獎金
    #[strum(to_string = "ForgetPassword")]
    ForgetPassword = 30, // 忘記密碼
    #[strum(to_string = "LoginPasswordReset")]
    LoginPasswordReset = 31, // 登入密碼重置
    #[strum(to_string = "LoginPasswordChange")]
    LoginPasswordChange = 32, // 登入密碼修改
    #[strum(to_string = "WithdrawPasswordSet")]
    WithdrawPasswordSet = 33, // 出金密碼設置
    #[strum(to_string = "WithdrawPasswordReset")]
    WithdrawPasswordReset = 34, // 出金密碼重置
}

impl From<NotifyEvent> for i64 {
    fn from(event_type: NotifyEvent) -> Self {
        event_type as i64
    }
}

impl TryFrom<i64> for NotifyEvent {
    type Error = KgsStatus;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(NotifyEvent::NormalInfo),
            2 => Ok(NotifyEvent::LoginAnomaly),
            3 => Ok(NotifyEvent::RegisterSuccess),
            4 => Ok(NotifyEvent::DepositSuccess),
            5 => Ok(NotifyEvent::WithdrawSuccess),
            6 => Ok(NotifyEvent::WithdrawFail),
            7 => Ok(NotifyEvent::KycVerifySuccess),
            8 => Ok(NotifyEvent::KycVerifyFail),
            9 => Ok(NotifyEvent::KycReverify),
            10 => Ok(NotifyEvent::CreditCardVerifySuccess),
            11 => Ok(NotifyEvent::CreditCardVerifyFail),
            12 => Ok(NotifyEvent::LoginWarning),
            13 => Ok(NotifyEvent::UpdateProfileSuccess),
            14 => Ok(NotifyEvent::LoginSuccess),
            15 => Ok(NotifyEvent::VerifyEmail),
            16 => Ok(NotifyEvent::VerifyPhone),
            17 => Ok(NotifyEvent::BackstageVerifyKyc),
            18 => Ok(NotifyEvent::BackstageVerifyWithdraw),
            19 => Ok(NotifyEvent::BackstageVerifyDeposit),
            20 => Ok(NotifyEvent::BackstageVerifyCreditCard),
            21 => Ok(NotifyEvent::NewEventOnline),
            22 => Ok(NotifyEvent::VipLevelUp),
            23 => Ok(NotifyEvent::BonusExpiration),
            24 => Ok(NotifyEvent::EventCompletion),
            25 => Ok(NotifyEvent::ReceiveTips),
            26 => Ok(NotifyEvent::GiveTips),
            27 => Ok(NotifyEvent::ReceiveBirthdayBonus),
            28 => Ok(NotifyEvent::ActivitySerialNumber),
            29 => Ok(NotifyEvent::ReceiveBonus),
            30 => Ok(NotifyEvent::ForgetPassword),
            31 => Ok(NotifyEvent::LoginPasswordReset),
            32 => Ok(NotifyEvent::LoginPasswordChange),
            33 => Ok(NotifyEvent::WithdrawPasswordSet),
            34 => Ok(NotifyEvent::WithdrawPasswordReset),
            _ => Err(KgsStatus::InvalidArgument),
        }
    }
}

impl NotifyEvent {
    pub fn to_id(&self) -> i64 {
        self.clone().into()
    }

    pub fn get_comment(&self) -> String {
        match self {
            Self::NormalInfo => "一般通知",
            Self::LoginAnomaly => "登入異常",
            Self::RegisterSuccess => "註冊成功",
            Self::DepositSuccess => "入金成功",
            Self::WithdrawSuccess => "出金成功",
            Self::WithdrawFail => "出金失敗",
            Self::KycVerifySuccess => "KYC 通過",
            Self::KycVerifyFail => "KYC 失敗",
            Self::KycReverify => "KYC 重新驗證",
            Self::CreditCardVerifySuccess => "信用卡驗證成功",
            Self::CreditCardVerifyFail => "信用卡驗證失敗",
            Self::LoginWarning => "登入警告 (密碼錯誤次數過多)",
            Self::UpdateProfileSuccess => "更新個人資料成功",
            Self::LoginSuccess => "登入成功",
            Self::VerifyEmail => "驗證信箱",
            Self::VerifyPhone => "驗證手機",
            Self::BackstageVerifyKyc => "後台審核 KYC",
            Self::BackstageVerifyDeposit => "後台審核出金",
            Self::BackstageVerifyWithdraw => "後台審核入金",
            Self::BackstageVerifyCreditCard => "後台審核信用卡",
            Self::NewEventOnline => "新活動上線",
            Self::VipLevelUp => "VIP等級提升",
            Self::BonusExpiration => "獎金到期",
            Self::EventCompletion => "活動完成",
            Self::ReceiveTips => "收到小費",
            Self::GiveTips => "給予小費",
            Self::ReceiveBirthdayBonus => "收到生日獎金",
            Self::ActivitySerialNumber => "收到活動序號",
            Self::ReceiveBonus => "收到獎金",
            Self::ForgetPassword => "忘記密碼",
            Self::LoginPasswordReset => "登入密碼重置",
            Self::LoginPasswordChange => "登入密碼修改",
            Self::WithdrawPasswordSet => "出金密碼設置",
            Self::WithdrawPasswordReset => "出金密碼重置",
        }
        .to_string()
    }

    pub fn get_platform(&self) -> Platform {
        match self {
            Self::NormalInfo => Platform::Frontend,
            Self::LoginAnomaly => Platform::Frontend,
            Self::RegisterSuccess => Platform::Frontend,
            Self::DepositSuccess => Platform::Frontend,
            Self::WithdrawSuccess => Platform::Frontend,
            Self::WithdrawFail => Platform::Frontend,
            Self::KycVerifySuccess => Platform::Frontend,
            Self::KycVerifyFail => Platform::Frontend,
            Self::KycReverify => Platform::Frontend,
            Self::CreditCardVerifySuccess => Platform::Frontend,
            Self::CreditCardVerifyFail => Platform::Frontend,
            Self::LoginWarning => Platform::Frontend,
            Self::UpdateProfileSuccess => Platform::Frontend,
            Self::LoginSuccess => Platform::Frontend,
            Self::VerifyEmail => Platform::Frontend,
            Self::VerifyPhone => Platform::Frontend,
            Self::BackstageVerifyKyc => Platform::Backstage,
            Self::BackstageVerifyWithdraw => Platform::Backstage,
            Self::BackstageVerifyDeposit => Platform::Backstage,
            Self::BackstageVerifyCreditCard => Platform::Backstage,
            Self::NewEventOnline => Platform::Frontend,
            Self::VipLevelUp => Platform::Frontend,
            Self::BonusExpiration => Platform::Frontend,
            Self::EventCompletion => Platform::Frontend,
            Self::ReceiveTips => Platform::Frontend,
            Self::GiveTips => Platform::Frontend,
            Self::ReceiveBirthdayBonus => Platform::Frontend,
            Self::ActivitySerialNumber => Platform::Frontend,
            Self::ReceiveBonus => Platform::Frontend,
            Self::ForgetPassword => Platform::Frontend,
            Self::LoginPasswordReset => Platform::Frontend,
            Self::LoginPasswordChange => Platform::Frontend,
            Self::WithdrawPasswordSet => Platform::Frontend,
            Self::WithdrawPasswordReset => Platform::Frontend,
        }
    }

    pub fn from_frontend_proto(
        event: protos::frontend_notify::SystemNotifyEvent,
    ) -> Result<Self, KgsStatus> {
        match event {
            protos::frontend_notify::SystemNotifyEvent::None => Err(KgsStatus::InvalidArgument),
            protos::frontend_notify::SystemNotifyEvent::NormalInfo => Ok(Self::NormalInfo),
            protos::frontend_notify::SystemNotifyEvent::LoginAnomaly => Ok(Self::LoginAnomaly),
            protos::frontend_notify::SystemNotifyEvent::RegisterSuccess => {
                Ok(Self::RegisterSuccess)
            }
            protos::frontend_notify::SystemNotifyEvent::DepositSuccess => Ok(Self::DepositSuccess),
            protos::frontend_notify::SystemNotifyEvent::WithdrawSuccess => {
                Ok(Self::WithdrawSuccess)
            }
            protos::frontend_notify::SystemNotifyEvent::WithdrawFail => Ok(Self::WithdrawFail),
            protos::frontend_notify::SystemNotifyEvent::KycVerifySuccess => {
                Ok(Self::KycVerifySuccess)
            }
            protos::frontend_notify::SystemNotifyEvent::KycVerifyFail => Ok(Self::KycVerifyFail),
            protos::frontend_notify::SystemNotifyEvent::KycReverify => Ok(Self::KycReverify),
            protos::frontend_notify::SystemNotifyEvent::CreditCardVerifySuccess => {
                Ok(Self::CreditCardVerifySuccess)
            }
            protos::frontend_notify::SystemNotifyEvent::CreditCardVerifyFail => {
                Ok(Self::CreditCardVerifyFail)
            }
            protos::frontend_notify::SystemNotifyEvent::LoginWarning => Ok(Self::LoginWarning),
            protos::frontend_notify::SystemNotifyEvent::UpdateProfileSuccess => {
                Ok(Self::UpdateProfileSuccess)
            }
            protos::frontend_notify::SystemNotifyEvent::LoginSuccess => Ok(Self::LoginSuccess),
            protos::frontend_notify::SystemNotifyEvent::VerifyEmail => Ok(Self::VerifyEmail),
            protos::frontend_notify::SystemNotifyEvent::VerifyPhone => Ok(Self::VerifyPhone),
            protos::frontend_notify::SystemNotifyEvent::NewEventOnline => Ok(Self::NewEventOnline),
            protos::frontend_notify::SystemNotifyEvent::VipLevelUp => Ok(Self::VipLevelUp),
            protos::frontend_notify::SystemNotifyEvent::BonusExpiration => {
                Ok(Self::BonusExpiration)
            }
            protos::frontend_notify::SystemNotifyEvent::EventCompletion => {
                Ok(Self::EventCompletion)
            }
            protos::frontend_notify::SystemNotifyEvent::ReceiveTips => Ok(Self::ReceiveTips),
            protos::frontend_notify::SystemNotifyEvent::GiveTips => Ok(Self::GiveTips),
            protos::frontend_notify::SystemNotifyEvent::ReceiveBirthdayBonus => {
                Ok(Self::ReceiveBirthdayBonus)
            }
            protos::frontend_notify::SystemNotifyEvent::ActivitySerialNumber => {
                Ok(Self::ActivitySerialNumber)
            }
            protos::frontend_notify::SystemNotifyEvent::ReceiveBonus => Ok(Self::ReceiveBonus),
            protos::frontend_notify::SystemNotifyEvent::ForgetPassword => Ok(Self::ForgetPassword),
            protos::frontend_notify::SystemNotifyEvent::LoginPasswordReset => {
                Ok(Self::LoginPasswordReset)
            }
            protos::frontend_notify::SystemNotifyEvent::LoginPasswordChange => {
                Ok(Self::LoginPasswordChange)
            }
            protos::frontend_notify::SystemNotifyEvent::WithdrawPasswordSet => {
                Ok(Self::WithdrawPasswordSet)
            }
            protos::frontend_notify::SystemNotifyEvent::WithdrawPasswordReset => {
                Ok(Self::WithdrawPasswordReset)
            }
        }
    }

    pub fn from_backstage_proto(
        event: protos::backstage_notify::BackStageNotifyEvent,
    ) -> Result<Self, KgsStatus> {
        match event {
            protos::backstage_notify::BackStageNotifyEvent::None => Err(KgsStatus::InvalidArgument),
            protos::backstage_notify::BackStageNotifyEvent::KycVerify => {
                Ok(Self::BackstageVerifyKyc)
            }
            protos::backstage_notify::BackStageNotifyEvent::WithdrawVerify => {
                Ok(Self::BackstageVerifyWithdraw)
            }
            protos::backstage_notify::BackStageNotifyEvent::DepositVerify => {
                Ok(Self::BackstageVerifyDeposit)
            }
            protos::backstage_notify::BackStageNotifyEvent::CreditCardVerify => {
                Ok(Self::BackstageVerifyCreditCard)
            }
        }
    }
}
