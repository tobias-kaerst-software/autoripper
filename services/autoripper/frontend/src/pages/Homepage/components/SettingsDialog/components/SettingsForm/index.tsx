import { zodResolver } from '@hookform/resolvers/zod';
import { FC, ReactNode, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';
import { useShallow } from 'zustand/react/shallow';

import { Form } from '$/components/common/ui/form';
import { DeviceSelection } from '$/pages/Homepage/components/SettingsDialog/components/SettingsForm/components/DeviceSelection';
import { EncodingPresetSelection } from '$/pages/Homepage/components/SettingsDialog/components/SettingsForm/components/EncodingPresetSelection';
import { MediaSelection } from '$/pages/Homepage/components/SettingsDialog/components/SettingsForm/components/MediaSelection';
import { ReloadButton } from '$/pages/Homepage/components/SettingsDialog/components/SettingsForm/components/ReloadButton';
import { MetadataFormValues, metadataFormSchema, useMediaStore } from '$/pages/Homepage/stores/useMediaStore';

import type { UseFormReturn } from 'react-hook-form';

export type MetadataFormControl = UseFormReturn<MetadataFormValues>;

interface Props {
  children: ReactNode;
  onSubmit?: (values: MetadataFormValues) => void;
}

export const SettingsForm: FC<Props> = ({ children, onSubmit }) => {
  const setMetadata = useMediaStore(useShallow((state) => state.setMetadata));

  const form = useForm<z.infer<typeof metadataFormSchema>>({
    resolver: zodResolver(metadataFormSchema),
    defaultValues: { type: 'movie', selectedSeason: 1, selectedEpisodes: [] },
  });

  const onSubmitHandler = (values: MetadataFormValues) => {
    setMetadata(values);
    onSubmit?.(values);
  };

  useEffect(() => {
    return useMediaStore.subscribe((state, prevState) => {
      if (!state.metadata && prevState.metadata) setTimeout(() => form.reset(), 0);
    });
  }, [form]);

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmitHandler)} className='space-y-8'>
        <div className='flex items-end gap-2'>
          <DeviceSelection form={form} />
          <EncodingPresetSelection form={form} />
          <ReloadButton form={form} />
        </div>

        <MediaSelection form={form} />

        {children}
      </form>
    </Form>
  );
};