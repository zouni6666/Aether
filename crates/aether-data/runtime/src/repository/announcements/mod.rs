mod memory;

pub use aether_data_contracts::repository::announcements::{
    AnnouncementListQuery, AnnouncementReadRepository, AnnouncementWriteRepository,
    CreateAnnouncementRecord, StoredAnnouncement, StoredAnnouncementPage, StoredUserAnnouncement,
    StoredUserAnnouncementPage, UpdateAnnouncementRecord, UserAnnouncementListQuery,
};
#[cfg(feature = "postgres")]
pub use aether_data_postgres::SqlxAnnouncementReadRepository;
pub use memory::InMemoryAnnouncementReadRepository;
